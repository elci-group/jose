//! Jose Phase I: deterministic, evidence-backed agent management primitives.
//!
//! This crate makes managerial decisions only. It does not invoke models or solve
//! the tasks assigned to them.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const SCHEMA_VERSION: u16 = 1;
pub const POLICY_VERSION: &str = "jose-policy-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(pub String);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerIdentity {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub quantisation: Option<String>,
    pub context_configuration: String,
    pub tool_configuration: String,
    pub reasoning_configuration: String,
    pub organisation_policy: String,
}

impl PlayerIdentity {
    /// Stable identity includes every execution-affecting configuration field.
    pub fn id(&self) -> PlayerId {
        let encoded = serde_json::to_vec(self).expect("identity serialization is infallible");
        let hash = stable_hash(&encoded);
        PlayerId(format!("{}:{}:{hash:016x}", self.provider, self.model))
    }
}

fn stable_hash(bytes: &[u8]) -> u64 {
    // FNV-1a is used for reproducible identifiers, not cryptographic integrity.
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Score(f64);

impl Score {
    pub fn new(value: f64) -> Result<Self, JoseError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(JoseError::InvalidScore(value));
        }
        Ok(Self(value))
    }
    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutonomyLevel {
    Observe,
    Classify,
    Verify,
    BoundedExecute,
    ExecuteRecover,
    PlanBounded,
    Coordinate,
    IndependentClose,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityProfile {
    pub player_id: PlayerId,
    pub role_scores: BTreeMap<String, Score>,
    pub instruction_sensitivity: Score,
    pub exposure_sensitivity: Score,
    pub anchoring_risk: Score,
    pub hallucination_risk: Score,
    pub scope_creep_risk: Score,
    pub cold_close_rate: Score,
    pub warm_close_rate: Score,
    pub autonomy_ceiling: AutonomyLevel,
    pub confidence: Score,
    pub sample_count: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub name: String,
    pub version: u32,
    pub purpose: String,
    pub permitted_actions: BTreeSet<String>,
    pub forbidden_actions: BTreeSet<String>,
    pub required_context: BTreeSet<String>,
    pub optional_context: BTreeSet<String>,
    pub deliverables: BTreeSet<String>,
    pub success_gates: BTreeSet<String>,
    pub escalation_conditions: BTreeSet<String>,
    pub expected_budget: u64,
    pub minimum_capability: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureClass {
    Public,
    Tactical,
    Positional,
    Developmental,
    Privileged,
    Withheld,
    Contaminated,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureObject {
    pub id: String,
    pub source: String,
    pub classification: ExposureClass,
    pub confidence_basis_points: u16,
    pub relevance: BTreeMap<String, u16>,
    pub contamination_risk_basis_points: u16,
    pub anchoring_risk_basis_points: u16,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub supersedes: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureAction {
    ExposeNow,
    ExposeLater,
    Withhold,
    Summarise,
    Transform,
    Challenge,
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureDecision {
    pub evidence_id: String,
    pub action: ExposureAction,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementAssignment {
    pub player_id: PlayerId,
    pub role: String,
    pub role_version: u32,
    pub objective: String,
    pub permitted_actions: BTreeSet<String>,
    pub forbidden_actions: BTreeSet<String>,
    pub visible_evidence: Vec<String>,
    pub withheld_evidence: Vec<String>,
    pub deliver_gates: BTreeSet<String>,
    pub escalation_conditions: BTreeSet<String>,
    pub expected_budget: u64,
    pub policy_version: String,
}

/// Rendered managerial brief plus only the evidence approved by exposure policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledInstruction {
    pub schema_version: u16,
    pub assignment: ManagementAssignment,
    pub instruction: String,
    pub exposed_context: Vec<ExposedEvidence>,
    pub exposure_decisions: Vec<ExposureDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposedEvidence {
    pub evidence_id: String,
    pub source: String,
    pub classification: ExposureClass,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub schema_version: u16,
    pub trial_id: String,
    /// Shared benchmark case identifier; cold and managed executions use the same value.
    pub pair_id: String,
    pub player_id: PlayerId,
    pub task_class: String,
    pub condition: TrialCondition,
    pub closed: bool,
    pub verifier_agreement: bool,
    pub false_close: bool,
    pub regression: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub latency_ms: u64,
    pub cost_microunits: u64,
    pub human_interventions: u32,
    pub role_version: Option<u32>,
    pub exposure_profile: String,
    pub instruction_profile: String,
    pub provocation: String,
    pub failure_category: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrialCondition {
    Cold,
    Managed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub task_class: String,
    pub player_id: PlayerId,
    pub cold_trials: u64,
    pub managed_trials: u64,
    pub cold_close_rate: f64,
    pub managed_close_rate: f64,
    pub relative_close_gain: Option<f64>,
    pub absolute_close_gain: f64,
    pub cold_cost_per_close_microunits: Option<f64>,
    pub managed_cost_per_close_microunits: Option<f64>,
    pub false_close_rate_managed: f64,
    pub regression_rate_managed: f64,
    pub mean_latency_delta_ms: Option<f64>,
    pub paired_trials: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayerState {
    Unscouted,
    Scouted,
    Profiled,
    Available,
    Conditioning,
    Ready,
    Active,
    Evaluation,
    Fatigued,
    Failed,
    Recovery,
    Quarantined,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub sequence: u64,
    pub timestamp: u64,
    pub trace_id: String,
    pub player: PlayerId,
    pub task: String,
    pub policy_version: String,
    pub evidence: Vec<String>,
    pub decision: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub identity: PlayerIdentity,
    pub state: PlayerState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Jose {
    pub schema_version: u16,
    pub policy_version: String,
    pub players: BTreeMap<PlayerId, PlayerRecord>,
    pub profiles: BTreeMap<PlayerId, CapabilityProfile>,
    pub roles: BTreeMap<String, RoleDefinition>,
    pub evidence: BTreeMap<String, ExposureObject>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub outcomes: BTreeMap<String, OutcomeRecord>,
}

#[derive(Debug, Error)]
pub enum JoseError {
    #[error("score must be finite and between 0 and 1, got {0}")]
    InvalidScore(f64),
    #[error("unknown player: {0}")]
    UnknownPlayer(PlayerId),
    #[error("unknown role: {0}")]
    UnknownRole(String),
    #[error("unknown evidence: {0}")]
    UnknownEvidence(String),
    #[error("invalid player state transition: {from:?} -> {to:?}")]
    InvalidTransition { from: PlayerState, to: PlayerState },
    #[error("role requirement not met: {0}")]
    RoleRequirement(String),
    #[error("duplicate evidence id: {0}")]
    DuplicateEvidence(String),
    #[error("duplicate outcome trial id: {0}")]
    DuplicateOutcome(String),
    #[error("invalid benchmark pair: {0}")]
    InvalidBenchmarkPair(String),
}

impl Default for Jose {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            policy_version: POLICY_VERSION.into(),
            players: BTreeMap::new(),
            profiles: BTreeMap::new(),
            roles: BTreeMap::new(),
            evidence: BTreeMap::new(),
            events: Vec::new(),
            outcomes: BTreeMap::new(),
        }
    }
}

impl Jose {
    pub fn register_player(&mut self, identity: PlayerIdentity) -> PlayerId {
        let id = identity.id();
        self.players.entry(id.clone()).or_insert(PlayerRecord {
            identity,
            state: PlayerState::Scouted,
        });
        id
    }

    pub fn add_role(&mut self, role: RoleDefinition) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn add_evidence(&mut self, evidence: ExposureObject) -> Result<(), JoseError> {
        if self.evidence.contains_key(&evidence.id) {
            return Err(JoseError::DuplicateEvidence(evidence.id));
        }
        self.evidence.insert(evidence.id.clone(), evidence);
        Ok(())
    }

    pub fn set_profile(&mut self, profile: CapabilityProfile) -> Result<(), JoseError> {
        if !self.players.contains_key(&profile.player_id) {
            return Err(JoseError::UnknownPlayer(profile.player_id));
        }
        self.profiles.insert(profile.player_id.clone(), profile);
        Ok(())
    }

    pub fn transition(
        &mut self,
        player: &PlayerId,
        to: PlayerState,
        task: &str,
        timestamp: u64,
        trace_id: &str,
    ) -> Result<(), JoseError> {
        let record = self
            .players
            .get_mut(player)
            .ok_or_else(|| JoseError::UnknownPlayer(player.clone()))?;
        if !transition_allowed(&record.state, &to) {
            return Err(JoseError::InvalidTransition {
                from: record.state.clone(),
                to,
            });
        }
        let from = record.state.clone();
        record.state = to.clone();
        self.push_event(
            timestamp,
            trace_id,
            player,
            task,
            vec![],
            format!("{from:?}->{to:?}"),
            "STATE_TRANSITION",
        );
        Ok(())
    }

    pub fn plan_exposure(
        &self,
        player: &PlayerId,
        role_name: &str,
        evidence_ids: &[String],
        now: u64,
    ) -> Result<Vec<ExposureDecision>, JoseError> {
        self.roles
            .get(role_name)
            .ok_or_else(|| JoseError::UnknownRole(role_name.into()))?;
        if !self.players.contains_key(player) {
            return Err(JoseError::UnknownPlayer(player.clone()));
        }
        let profile = self.profiles.get(player);
        let mut decisions = Vec::new();
        for id in evidence_ids {
            let item = self
                .evidence
                .get(id)
                .ok_or_else(|| JoseError::UnknownEvidence(id.clone()))?;
            let action = if item.expires_at.is_some_and(|expiry| expiry <= now)
                || item.classification == ExposureClass::Expired
            {
                (ExposureAction::Discard, "EVIDENCE_EXPIRED")
            } else if item.classification == ExposureClass::Contaminated {
                (ExposureAction::Withhold, "CONTAMINATED_EVIDENCE")
            } else if item.classification == ExposureClass::Withheld
                || item.classification == ExposureClass::Privileged
            {
                (ExposureAction::Withhold, "POLICY_WITHHELD")
            } else if !item.relevance.contains_key(role_name) {
                (ExposureAction::Discard, "IRRELEVANT_TO_ROLE")
            } else {
                let anchor_limit = profile
                    .map(|p| (p.anchoring_risk.get() * 10_000.0) as u16)
                    .unwrap_or(5_000);
                if item.anchoring_risk_basis_points > anchor_limit
                    && !role_name.to_ascii_lowercase().contains("independent")
                {
                    (ExposureAction::Withhold, "ANCHORING_RISK")
                } else if item.classification == ExposureClass::Developmental {
                    (ExposureAction::ExposeLater, "DEVELOPMENTAL_CONDITIONING")
                } else {
                    (ExposureAction::ExposeNow, "ROLE_RELEVANT_EVIDENCE")
                }
            };
            decisions.push(ExposureDecision {
                evidence_id: id.clone(),
                action: action.0,
                reason_code: action.1.into(),
            });
        }
        Ok(decisions)
    }

    pub fn compile_assignment(
        &self,
        player: &PlayerId,
        role_name: &str,
        objective: &str,
        evidence_ids: &[String],
        now: u64,
    ) -> Result<ManagementAssignment, JoseError> {
        if !self.players.contains_key(player) {
            return Err(JoseError::UnknownPlayer(player.clone()));
        }
        let role = self
            .roles
            .get(role_name)
            .ok_or_else(|| JoseError::UnknownRole(role_name.into()))?;
        let profile = self.profiles.get(player).ok_or_else(|| {
            JoseError::RoleRequirement("player has no evidence-backed profile".into())
        })?;
        let score = profile.role_scores.get(role_name).ok_or_else(|| {
            JoseError::RoleRequirement(format!("missing score for role {role_name}"))
        })?;
        if ((score.get() * 100.0).round() as u8) < role.minimum_capability {
            return Err(JoseError::RoleRequirement(format!(
                "{role_name} requires {}, profile provides {:.2}",
                role.minimum_capability,
                score.get()
            )));
        }
        let decisions = self.plan_exposure(player, role_name, evidence_ids, now)?;
        let visible_evidence = decisions
            .iter()
            .filter(|d| {
                matches!(
                    d.action,
                    ExposureAction::ExposeNow
                        | ExposureAction::Summarise
                        | ExposureAction::Transform
                )
            })
            .map(|d| d.evidence_id.clone())
            .collect();
        let withheld_evidence = decisions
            .iter()
            .filter(|d| matches!(d.action, ExposureAction::Withhold | ExposureAction::Discard))
            .map(|d| d.evidence_id.clone())
            .collect();
        Ok(ManagementAssignment {
            player_id: player.clone(),
            role: role.name.clone(),
            role_version: role.version,
            objective: objective.into(),
            permitted_actions: role.permitted_actions.clone(),
            forbidden_actions: role.forbidden_actions.clone(),
            visible_evidence,
            withheld_evidence,
            deliver_gates: role.success_gates.clone(),
            escalation_conditions: role.escalation_conditions.clone(),
            expected_budget: role.expected_budget,
            policy_version: self.policy_version.clone(),
        })
    }

    /// Compile an assignment into a deterministic prompt brief and approved context.
    /// The output deliberately contains no incumbent solution or full transcript.
    pub fn compile_instruction(
        &self,
        player: &PlayerId,
        role_name: &str,
        objective: &str,
        evidence_ids: &[String],
        now: u64,
    ) -> Result<CompiledInstruction, JoseError> {
        let assignment =
            self.compile_assignment(player, role_name, objective, evidence_ids, now)?;
        let decisions = self.plan_exposure(player, role_name, evidence_ids, now)?;
        let exposed_context = decisions
            .iter()
            .filter(|decision| decision.action == ExposureAction::ExposeNow)
            .map(|decision| {
                let evidence = &self.evidence[&decision.evidence_id];
                ExposedEvidence {
                    evidence_id: evidence.id.clone(),
                    source: evidence.source.clone(),
                    classification: evidence.classification,
                    content: evidence.content.clone(),
                }
            })
            .collect::<Vec<_>>();
        let instruction = render_instruction(&assignment, &exposed_context);
        Ok(CompiledInstruction {
            schema_version: SCHEMA_VERSION,
            assignment,
            instruction,
            exposed_context,
            exposure_decisions: decisions,
        })
    }

    /// Record externally verified execution results. Jose does not run or judge the task.
    pub fn record_outcome(&mut self, outcome: OutcomeRecord) -> Result<(), JoseError> {
        if !self.players.contains_key(&outcome.player_id) {
            return Err(JoseError::UnknownPlayer(outcome.player_id));
        }
        if self.outcomes.contains_key(&outcome.trial_id) {
            return Err(JoseError::DuplicateOutcome(outcome.trial_id));
        }
        let player = outcome.player_id.clone();
        let task = outcome.task_class.clone();
        let trial_id = outcome.trial_id.clone();
        let decision = format!("OUTCOME:{}:{}", outcome.condition_label(), outcome.closed);
        self.outcomes.insert(trial_id.clone(), outcome);
        self.push_event(
            0,
            &trial_id,
            &player,
            &task,
            vec![],
            decision,
            "OUTCOME_RECORDED",
        );
        Ok(())
    }

    /// Summarise cold/managed paired trials with no inferential claim beyond observed data.
    pub fn benchmark_summary(
        &self,
        task_class: &str,
        player: &PlayerId,
    ) -> Result<BenchmarkSummary, JoseError> {
        if !self.players.contains_key(player) {
            return Err(JoseError::UnknownPlayer(player.clone()));
        }
        let trials = self
            .outcomes
            .values()
            .filter(|trial| &trial.player_id == player && trial.task_class == task_class)
            .collect::<Vec<_>>();
        let cold = trials
            .iter()
            .filter(|trial| trial.condition == TrialCondition::Cold)
            .copied()
            .collect::<Vec<_>>();
        let managed = trials
            .iter()
            .filter(|trial| trial.condition == TrialCondition::Managed)
            .copied()
            .collect::<Vec<_>>();
        if cold.is_empty() || managed.is_empty() {
            return Err(JoseError::InvalidBenchmarkPair(format!(
                "need at least one cold and one managed trial for {task_class}"
            )));
        }
        let rate = |items: &[&OutcomeRecord]| {
            items
                .iter()
                .filter(|trial| trial.closed && trial.verifier_agreement && !trial.false_close)
                .count() as f64
                / items.len() as f64
        };
        let cold_rate = rate(&cold);
        let managed_rate = rate(&managed);
        let cost_per_close = |items: &[&OutcomeRecord]| {
            let successes = items
                .iter()
                .filter(|trial| trial.closed && trial.verifier_agreement && !trial.false_close)
                .count();
            (successes > 0).then(|| {
                items
                    .iter()
                    .map(|trial| trial.cost_microunits as f64)
                    .sum::<f64>()
                    / successes as f64
            })
        };
        let cold_mean_latency = cold
            .iter()
            .map(|trial| trial.latency_ms as f64)
            .sum::<f64>()
            / cold.len() as f64;
        let managed_mean_latency = managed
            .iter()
            .map(|trial| trial.latency_ms as f64)
            .sum::<f64>()
            / managed.len() as f64;
        let paired_trials = cold
            .iter()
            .filter(|cold_trial| {
                managed
                    .iter()
                    .any(|managed_trial| managed_trial.pair_id == cold_trial.pair_id)
            })
            .count() as u64;
        Ok(BenchmarkSummary {
            task_class: task_class.into(),
            player_id: player.clone(),
            cold_trials: cold.len() as u64,
            managed_trials: managed.len() as u64,
            cold_close_rate: cold_rate,
            managed_close_rate: managed_rate,
            relative_close_gain: (cold_rate > 0.0)
                .then_some((managed_rate - cold_rate) / cold_rate),
            absolute_close_gain: managed_rate - cold_rate,
            cold_cost_per_close_microunits: cost_per_close(&cold),
            managed_cost_per_close_microunits: cost_per_close(&managed),
            false_close_rate_managed: managed.iter().filter(|trial| trial.false_close).count()
                as f64
                / managed.len() as f64,
            regression_rate_managed: managed.iter().filter(|trial| trial.regression).count() as f64
                / managed.len() as f64,
            mean_latency_delta_ms: Some(managed_mean_latency - cold_mean_latency),
            paired_trials,
        })
    }

    fn push_event(
        &mut self,
        timestamp: u64,
        trace_id: &str,
        player: &PlayerId,
        task: &str,
        evidence: Vec<String>,
        decision: String,
        reason_code: &str,
    ) {
        self.events.push(Event {
            sequence: self.events.len() as u64,
            timestamp,
            trace_id: trace_id.into(),
            player: player.clone(),
            task: task.into(),
            policy_version: self.policy_version.clone(),
            evidence,
            decision,
            reason_code: reason_code.into(),
        });
    }

    pub fn replay(events: &[Event]) -> Result<BTreeMap<PlayerId, PlayerState>, JoseError> {
        let mut state = BTreeMap::new();
        for event in events {
            let (from, to) =
                event
                    .decision
                    .split_once("->")
                    .ok_or_else(|| JoseError::InvalidTransition {
                        from: PlayerState::Unscouted,
                        to: PlayerState::Unscouted,
                    })?;
            let from = parse_state(from).ok_or_else(|| JoseError::InvalidTransition {
                from: PlayerState::Unscouted,
                to: PlayerState::Unscouted,
            })?;
            let to = parse_state(to).ok_or_else(|| JoseError::InvalidTransition {
                from: PlayerState::Unscouted,
                to: PlayerState::Unscouted,
            })?;
            let current = state.entry(event.player.clone()).or_insert(from.clone());
            if *current != from || !transition_allowed(current, &to) {
                return Err(JoseError::InvalidTransition {
                    from: current.clone(),
                    to,
                });
            }
            *current = to;
        }
        Ok(state)
    }
}

fn parse_state(value: &str) -> Option<PlayerState> {
    Some(match value {
        "Unscouted" => PlayerState::Unscouted,
        "Scouted" => PlayerState::Scouted,
        "Profiled" => PlayerState::Profiled,
        "Available" => PlayerState::Available,
        "Conditioning" => PlayerState::Conditioning,
        "Ready" => PlayerState::Ready,
        "Active" => PlayerState::Active,
        "Evaluation" => PlayerState::Evaluation,
        "Fatigued" => PlayerState::Fatigued,
        "Failed" => PlayerState::Failed,
        "Recovery" => PlayerState::Recovery,
        "Quarantined" => PlayerState::Quarantined,
        "Retired" => PlayerState::Retired,
        _ => return None,
    })
}

impl TrialCondition {
    fn label(self) -> &'static str {
        match self {
            Self::Cold => "COLD",
            Self::Managed => "MANAGED",
        }
    }
}

impl OutcomeRecord {
    fn condition_label(&self) -> &'static str {
        self.condition.label()
    }
}

fn render_instruction(assignment: &ManagementAssignment, evidence: &[ExposedEvidence]) -> String {
    let mut lines = vec![
        format!("ROLE\n{} (v{})", assignment.role, assignment.role_version),
        format!("OBJECTIVE\n{}", assignment.objective),
        format!("PERMITTED\n{}", join_or_none(&assignment.permitted_actions)),
        format!("FORBIDDEN\n{}", join_or_none(&assignment.forbidden_actions)),
        format!(
            "VISIBLE EVIDENCE\n{}",
            if evidence.is_empty() {
                "none".into()
            } else {
                evidence
                    .iter()
                    .map(|item| format!("[{}] {}", item.evidence_id, item.content))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        ),
        format!("DELIVER GATES\n{}", join_or_none(&assignment.deliver_gates)),
        format!(
            "ESCALATE IF\n{}",
            join_or_none(&assignment.escalation_conditions)
        ),
        format!("BUDGET\n{} tokens", assignment.expected_budget),
    ];
    lines.push(format!("POLICY\n{}", assignment.policy_version));
    lines.join("\n\n")
}

fn join_or_none(values: &BTreeSet<String>) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}

fn transition_allowed(from: &PlayerState, to: &PlayerState) -> bool {
    use PlayerState::*;
    if matches!(to, Quarantined | Retired) {
        return !matches!(from, Retired);
    }
    matches!(
        (from, to),
        (Unscouted, Scouted)
            | (Scouted, Profiled)
            | (Profiled, Available)
            | (Available, Conditioning)
            | (Available, Ready)
            | (Conditioning, Ready)
            | (Ready, Active)
            | (Active, Evaluation)
            | (Active, Fatigued)
            | (Active, Failed)
            | (Fatigued, Recovery)
            | (Failed, Recovery)
            | (Recovery, Ready)
            | (Evaluation, Available)
    )
}

/// Create the initial regression-closer role used by the CLI and fixtures.
pub fn regression_closer_role() -> RoleDefinition {
    RoleDefinition {
        name: "regression_closer".into(),
        version: 1,
        purpose: "Repair a narrowly scoped regression and satisfy deterministic gates.".into(),
        permitted_actions: [
            "inspect_touched_files",
            "edit_implementation",
            "run_target_tests",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        forbidden_actions: [
            "public_api_change",
            "dependency_replacement",
            "unrelated_refactor",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        required_context: ["failing_test", "accepted_invariants"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        optional_context: ["modified_files", "platform_abstraction"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        deliverables: ["patch", "test_results"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        success_gates: [
            "target_tests_pass",
            "parity_tests_unchanged",
            "unrelated_diff_zero",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        escalation_conditions: ["public_interface_change_required"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        expected_budget: 20_000,
        minimum_capability: 60,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn identity(model: &str) -> PlayerIdentity {
        PlayerIdentity {
            provider: "mock".into(),
            model: model.into(),
            endpoint: "local".into(),
            quantisation: None,
            context_configuration: "default".into(),
            tool_configuration: "read-write".into(),
            reasoning_configuration: "standard".into(),
            organisation_policy: "test".into(),
        }
    }
    fn score(v: f64) -> Score {
        Score::new(v).unwrap()
    }
    fn profile(id: PlayerId) -> CapabilityProfile {
        CapabilityProfile {
            player_id: id,
            role_scores: [("regression_closer".into(), score(0.9))]
                .into_iter()
                .collect(),
            instruction_sensitivity: score(0.8),
            exposure_sensitivity: score(0.7),
            anchoring_risk: score(0.2),
            hallucination_risk: score(0.1),
            scope_creep_risk: score(0.1),
            cold_close_rate: score(0.4),
            warm_close_rate: score(0.8),
            autonomy_ceiling: AutonomyLevel::BoundedExecute,
            confidence: score(0.9),
            sample_count: 20,
            updated_at: 1,
        }
    }
    #[test]
    fn identity_changes_with_execution_configuration() {
        let a = identity("m");
        let mut b = a.clone();
        b.tool_configuration = "readonly".into();
        assert_ne!(a.id(), b.id());
    }
    #[test]
    fn assignment_is_deterministic_and_applies_exposure_policy() {
        let mut jose = Jose::default();
        let id = jose.register_player(identity("mock"));
        jose.add_role(regression_closer_role());
        jose.set_profile(profile(id.clone())).unwrap();
        jose.add_evidence(ExposureObject {
            id: "test".into(),
            source: "fixture".into(),
            classification: ExposureClass::Tactical,
            confidence_basis_points: 9900,
            relevance: [("regression_closer".into(), 10000)].into_iter().collect(),
            contamination_risk_basis_points: 0,
            anchoring_risk_basis_points: 100,
            created_at: 1,
            expires_at: None,
            supersedes: vec![],
            content: "failing test".into(),
        })
        .unwrap();
        let plan = jose
            .compile_assignment(
                &id,
                "regression_closer",
                "Fix regression",
                &["test".into()],
                2,
            )
            .unwrap();
        assert_eq!(plan.visible_evidence, vec!["test"]);
        assert_eq!(
            plan,
            jose.compile_assignment(
                &id,
                "regression_closer",
                "Fix regression",
                &["test".into()],
                2
            )
            .unwrap()
        );
    }
    #[test]
    fn transition_replay_reaches_same_state() {
        let mut jose = Jose::default();
        let id = jose.register_player(identity("mock"));
        jose.transition(&id, PlayerState::Profiled, "task", 1, "trace")
            .unwrap();
        jose.transition(&id, PlayerState::Available, "task", 2, "trace")
            .unwrap();
        jose.transition(&id, PlayerState::Conditioning, "task", 3, "trace")
            .unwrap();
        jose.transition(&id, PlayerState::Ready, "task", 4, "trace")
            .unwrap();
        assert_eq!(Jose::replay(&jose.events).unwrap()[&id], PlayerState::Ready);
    }
    #[test]
    fn exposure_fails_closed_on_contamination_and_expiry() {
        let mut jose = Jose::default();
        let id = jose.register_player(identity("mock"));
        jose.add_role(regression_closer_role());
        jose.set_profile(profile(id.clone())).unwrap();
        for (id, class, expiry) in [
            ("bad", ExposureClass::Contaminated, None),
            ("old", ExposureClass::Tactical, Some(2)),
        ] {
            jose.add_evidence(ExposureObject {
                id: id.into(),
                source: "fixture".into(),
                classification: class,
                confidence_basis_points: 5000,
                relevance: [("regression_closer".into(), 10000)].into_iter().collect(),
                contamination_risk_basis_points: 9000,
                anchoring_risk_basis_points: 0,
                created_at: 0,
                expires_at: expiry,
                supersedes: vec![],
                content: "untrusted".into(),
            })
            .unwrap();
        }
        let plan = jose
            .plan_exposure(&id, "regression_closer", &["bad".into(), "old".into()], 3)
            .unwrap();
        assert_eq!(plan[0].action, ExposureAction::Withhold);
        assert_eq!(plan[1].action, ExposureAction::Discard);
    }

    fn ready_jose() -> (Jose, PlayerId) {
        let mut jose = Jose::default();
        let id = jose.register_player(identity("bench-model"));
        jose.add_role(regression_closer_role());
        jose.set_profile(profile(id.clone())).unwrap();
        jose.add_evidence(ExposureObject {
            id: "failing-test".into(),
            source: "verifier".into(),
            classification: ExposureClass::Tactical,
            confidence_basis_points: 9900,
            relevance: [("regression_closer".into(), 10000)].into_iter().collect(),
            contamination_risk_basis_points: 0,
            anchoring_risk_basis_points: 100,
            created_at: 1,
            expires_at: None,
            supersedes: vec![],
            content: "The Windows path regression test fails.".into(),
        })
        .unwrap();
        jose.add_evidence(ExposureObject {
            id: "incumbent-patch".into(),
            source: "prior-agent".into(),
            classification: ExposureClass::Contaminated,
            confidence_basis_points: 4000,
            relevance: [("regression_closer".into(), 10000)].into_iter().collect(),
            contamination_risk_basis_points: 8000,
            anchoring_risk_basis_points: 9000,
            created_at: 1,
            expires_at: None,
            supersedes: vec![],
            content: "Use this exact patch.".into(),
        })
        .unwrap();
        (jose, id)
    }

    #[test]
    fn compiled_instruction_is_stable_and_excludes_withheld_context() {
        let (jose, id) = ready_jose();
        let evidence = vec!["failing-test".into(), "incumbent-patch".into()];
        let first = jose
            .compile_instruction(
                &id,
                "regression_closer",
                "Restore Windows path compatibility",
                &evidence,
                2,
            )
            .unwrap();
        let second = jose
            .compile_instruction(
                &id,
                "regression_closer",
                "Restore Windows path compatibility",
                &evidence,
                2,
            )
            .unwrap();
        assert_eq!(first, second);
        assert!(
            first
                .instruction
                .contains("Restore Windows path compatibility")
        );
        assert_eq!(first.exposed_context.len(), 1);
        assert_eq!(first.exposed_context[0].evidence_id, "failing-test");
        assert!(first.instruction.contains("incumbent-patch") == false);
        assert_eq!(first.exposure_decisions[1].action, ExposureAction::Withhold);
    }

    fn outcome(
        player: PlayerId,
        trial: &str,
        pair: &str,
        condition: TrialCondition,
        closed: bool,
        cost: u64,
        latency: u64,
    ) -> OutcomeRecord {
        OutcomeRecord {
            schema_version: SCHEMA_VERSION,
            trial_id: trial.into(),
            pair_id: pair.into(),
            player_id: player,
            task_class: "bounded_regression".into(),
            condition,
            closed,
            verifier_agreement: closed,
            false_close: false,
            regression: false,
            input_tokens: 100,
            output_tokens: 20,
            latency_ms: latency,
            cost_microunits: cost,
            human_interventions: 0,
            role_version: (condition == TrialCondition::Managed).then_some(1),
            exposure_profile: if condition == TrialCondition::Managed {
                "bounded"
            } else {
                "cold"
            }
            .into(),
            instruction_profile: if condition == TrialCondition::Managed {
                "regression-closer-v1"
            } else {
                "standard"
            }
            .into(),
            provocation: "none".into(),
            failure_category: (!closed).then_some("CAPABILITY_LIMIT".into()),
        }
    }

    #[test]
    fn paired_benchmark_reports_managed_gain_and_economics() {
        let (mut jose, id) = ready_jose();
        jose.record_outcome(outcome(
            id.clone(),
            "cold-a",
            "case-a",
            TrialCondition::Cold,
            false,
            100,
            1200,
        ))
        .unwrap();
        jose.record_outcome(outcome(
            id.clone(),
            "managed-a",
            "case-a",
            TrialCondition::Managed,
            true,
            70,
            900,
        ))
        .unwrap();
        jose.record_outcome(outcome(
            id.clone(),
            "cold-b",
            "case-b",
            TrialCondition::Cold,
            true,
            100,
            1000,
        ))
        .unwrap();
        jose.record_outcome(outcome(
            id.clone(),
            "managed-b",
            "case-b",
            TrialCondition::Managed,
            true,
            70,
            800,
        ))
        .unwrap();
        let summary = jose.benchmark_summary("bounded_regression", &id).unwrap();
        assert_eq!(summary.paired_trials, 2);
        assert_eq!(summary.cold_close_rate, 0.5);
        assert_eq!(summary.managed_close_rate, 1.0);
        assert_eq!(summary.absolute_close_gain, 0.5);
        assert_eq!(summary.cold_cost_per_close_microunits, Some(200.0));
        assert_eq!(summary.managed_cost_per_close_microunits, Some(70.0));
        assert_eq!(summary.mean_latency_delta_ms, Some(-250.0));
    }

    #[test]
    fn outcome_ingestion_rejects_duplicates_and_benchmark_needs_both_conditions() {
        let (mut jose, id) = ready_jose();
        let record = outcome(
            id.clone(),
            "only",
            "case",
            TrialCondition::Cold,
            false,
            5,
            10,
        );
        jose.record_outcome(record.clone()).unwrap();
        assert!(matches!(
            jose.record_outcome(record),
            Err(JoseError::DuplicateOutcome(_))
        ));
        assert!(matches!(
            jose.benchmark_summary("bounded_regression", &id),
            Err(JoseError::InvalidBenchmarkPair(_))
        ));
    }
}
