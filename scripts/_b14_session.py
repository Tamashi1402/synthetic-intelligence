import io, re

p = 'crates/si-session/src/lib.rs'
s = open(p, encoding='utf-8').read()

# 1. install_arc_config + accessors beside install_launch_constants
old = """    pub fn install_launch_constants(&mut self, params: Params, walk: WalkConstants) {
        self.params = params;
        self.walk = walk;
    }
"""
new = """    pub fn install_launch_constants(&mut self, params: Params, walk: WalkConstants) {
        self.params = params;
        self.walk = walk;
    }

    /// engineering/12 §3.2: the launch channel's arc half. A Ruling row installs
    /// through the same door a SPECIES row does; the class check lives in the
    /// harness, and `Temperament`/`Law`/`Derived` stay refused there.
    pub fn install_arc_config(&mut self, arc: ArcConfig) {
        self.arc = arc;
        self.crystallizer = si_crystallize::Crystallizer::new(arc.crystallize_config());
    }

    pub fn arc(&self) -> ArcConfig {
        self.arc
    }

    /// OD-98's audit trail: one line per candidate transition.
    pub fn crystallize_lines(&self) -> &[serde_json::Value] {
        &self.crystallize_lines
    }

    /// 12 §2.6: the frame scratch each thought exported. Hashed nowhere.
    pub fn frame_logs(&self) -> &[serde_json::Value] {
        &self.frame_logs
    }

    /// `mindlog.json`'s `nomination_views{}`: the candidate table as the log
    /// shows it. Session RAM, never a durable byte.
    pub fn nomination_views(&self) -> serde_json::Value {
        let mut views = serde_json::Map::new();
        for (hash, candidate) in self.crystallizer.table() {
            views.insert(
                format!("{hash:016x}"),
                serde_json::json!({
                    "members": candidate.members,
                    "shared_fillers": candidate.shared_fillers,
                    "recurrence": candidate.recurrence,
                    "frames_seen": candidate.frames_seen,
                    "first_seen_t": candidate.first_seen_t,
                    "last_seen_t": candidate.last_seen_t,
                    "nominated_at": candidate.nominated_at,
                }),
            );
        }
        serde_json::Value::Object(views)
    }
"""
assert old in s
s = s.replace(old, new, 1)

# 2. walk_read: observe after record_thought
old = """        self.record_thought(t, thought_id, &retrieval.thought);
        Ok(retrieval)
    }
"""
new = """        self.record_thought(t, thought_id, &retrieval.thought);
        // 02 §11 step 7a (engineering/12 §4.1): observation is after settle and
        // before commit — a detector that ran earlier would nominate on a
        // half-formed thought, one that ran later could read its own minted row.
        self.observe_frame(t, thought_id, &retrieval.thought)?;
        Ok(retrieval)
    }

    /// Step 7a: export the frame log (12 §2.6) and, when `CRYSTALLIZE` is on,
    /// ask the detector. Its only output is a `Nomination` — the ask below is
    /// this session's ordinary `questions` write through the ordinary queue,
    /// never a row the detector could reach (12 §1.2 win 5).
    fn observe_frame(&mut self, t: i32, thought_id: u32, thought: &Thought) -> Result<()> {
        self.frame_logs.push(frame_log_json(thought_id, &thought.frame_log));
        if !self.arc.crystallize {
            return Ok(());
        }
        let stage = gate::stage_of(&self.store)?;
        let observation = self.crystallizer.observe(&thought.frame_log, &self.store, t)?;
        for event in &observation.events {
            self.crystallize_lines.push(serde_json::json!({
                "t": event.t,
                "thought": thought_id,
                "transition": transition_label(event.transition),
                "members": event.members,
                "recurrence": event.recurrence,
                "candidates": observation.candidates,
            }));
        }
        for nomination in &observation.nominations {
            self.nomination_ask(t, thought_id, stage, nomination)?;
        }
        Ok(())
    }

    /// §22.4 / 12 §2.4: an ordinary ask on the ordinary queue. `origin = 2`
    /// decides the lane, `arousal_at_ask = 255` is the row's own I8 assertion,
    /// and `text_ref` is an already-interned member surface so the hashed
    /// interner cannot grow from a detector's output (05 §23).
    fn nomination_ask(
        &mut self,
        t: i32,
        thought_id: u32,
        stage: u8,
        nomination: &si_crystallize::Nomination,
    ) -> Result<()> {
        let anchor = nomination
            .members
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("a nomination names no member"))?;
        let budget = drive::budget(&self.params);
        let emitted = drive::emit(
            &[drive::Ranked {
                gap: drive::Gap {
                    slot: anchor,
                    salience_q12: budget.salience_floor,
                    last_pulsed_at: t,
                    usage_q12: Q12_ONE,
                },
                weight_q12: budget.salience_floor,
            }],
            &self.params,
        );
        if emitted.is_empty() {
            return Ok(());
        }
        if let Some(existing) =
            gate::ask_at(&self.store, t, si_crystallize::PATTERN_NAMELESS, anchor, 0)?
        {
            if existing.status == gate::Q_OPEN {
                return Ok(());
            }
        }
        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source: Source::Owner,
            stage,
            a_formation: self.a.a(),
            thought_id,
            signal_ref: t as u32,
        };
        let question_id = gate::append_question(
            &mut self.store,
            &QuestionSpec {
                pattern_id: si_crystallize::PATTERN_NAMELESS,
                gap_kind: si_crystallize::GAP_CLUSTER_NOMINATION,
                origin: si_crystallize::ASK_ORIGIN_NOMINATION,
                key_point: anchor,
                target_slot: 0,
                text_ref: nomination.text_ref,
                prov_step: t,
                arousal_at_ask: si_core::record::AROUSAL_NONE,
            },
            &ctx,
        )?;
        self.stats.asks += 1;
        self.last_output_row = Some((SHELF_QUESTIONS, question_id));
        let members: Vec<String> = nomination
            .members
            .iter()
            .map(|point| point.to_string())
            .collect();
        self.crystallize_lines.push(serde_json::json!({
            "t": t,
            "thought": thought_id,
            "transition": "ask",
            "members": nomination.members,
            "recurrence": nomination.recurrence,
            "question": question_id,
            "kind": si_crystallize::GAP_CLUSTER_NOMINATION,
            "origin": si_crystallize::ASK_ORIGIN_NOMINATION,
        }));
        let detail = format!(
            "{{\"kind\":\"cluster-nomination\",\"question\":{question_id},\"origin\":2,\"members\":[{}]}}",
            members.join(",")
        );
        self.log(t, 0, EV_ASK, detail);
        Ok(())
    }
"""
assert old in s, 'walk_read anchor'
s = s.replace(old, new, 1)

# 3. idle_tick: never revisit an origin-2 ask
old = """        let open = gate::open_asks(&self.store, t)?;
        for ask in open {
            let pattern = PatternId::from_index(ask.pattern_id as u8).unwrap_or(PatternId::Undecoded);"""
new = """        let open = gate::open_asks(&self.store, t)?;
        for ask in open {
            // OD-128 / 12 §2.4: an `origin = 2` nomination ask answers only in
            // the owner lane that mints its point (B17). The revisit is not that
            // lane: letting it resolve the ask would have SI answer its own
            // nomination from the very rows it noticed. The ask stays open, so
            // it keeps the agenda non-empty and suppresses ignition exactly as
            // any other unanswered ask does.
            if ask.origin() == gate::ASK_ORIGIN_NOMINATION {
                continue;
            }
            let pattern = PatternId::from_index(ask.pattern_id as u8).unwrap_or(PatternId::Undecoded);"""
assert old in s, 'idle anchor'
s = s.replace(old, new, 1)

# 4. frame-log/transition helpers at the end of the file (before SessionParser)
old = """impl SessionParser {"""
new = """/// 12 §2.6: the frame scratch as the mind log shows it. `role` and `round` are
/// the log's whole content; a ghost carries no name (docs/22 §22.2).
fn frame_log_json(thought_id: u32, log: &si_core::frame::FrameLog) -> serde_json::Value {
    let lights: Vec<serde_json::Value> = log
        .lights
        .iter()
        .map(|light| {
            serde_json::json!({
                "point": light.point,
                "role": light.role.label(),
                "edge_kind": light.edge_kind,
                "round": light.round,
                "primed": light.primed,
            })
        })
        .collect();
    let ghosts: Vec<serde_json::Value> = log
        .ghosts
        .iter()
        .map(|ghost| {
            serde_json::json!({
                "members": ghost.members,
                "formed_in_frame": ghost.formed_in_frame,
                "frame_seed": ghost.frame_seed,
            })
        })
        .collect();
    let role_graphs: Vec<serde_json::Value> = log
        .role_graphs
        .iter()
        .map(|graph| {
            serde_json::json!({
                "roles": graph.roles,
                "traversed_kinds": graph.traversed_kinds,
                "filler_sets": graph.filler_sets,
            })
        })
        .collect();
    serde_json::json!({
        "thought": thought_id,
        "cold": log.is_cold(),
        "lights": lights,
        "ghosts": ghosts,
        "role_graphs": role_graphs,
    })
}

fn transition_label(transition: si_crystallize::Transition) -> &'static str {
    use si_crystallize::Transition as T;
    match transition {
        T::WarmFrame => "warm-frame",
        T::TooFewMembers => "too-few-members",
        T::NoSharedFiller => "no-shared-filler",
        T::AlreadyBound => "already-bound",
        T::Opened => "open",
        T::Recurred => "recurrence",
        T::InsideDelta => "inside-delta",
        T::Nominated => "nominate",
    }
}

impl SessionParser {"""
assert old in s, 'parser anchor'
s = s.replace(old, new, 1)

open(p, 'w', encoding='utf-8').write(s)
print('ok')
