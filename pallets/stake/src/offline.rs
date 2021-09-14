// SPDX-License-Identifier: Apache-2.0

use codec::{Decode, Encode};
use sp_runtime::{Perbill, RuntimeDebug};
use sp_std::prelude::*;

#[derive(Clone, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
pub enum OffenceKind {
    Deal,
    Attest,
    Provider,
}

pub type Kind = [u8; 16];

pub trait AutomataOffence<Offender> {
    const ID: Kind;
    type SpecialId: Clone + codec::Codec + Ord;
    fn kind(&self) -> Kind;
    fn offender(&self) -> Offender;
    fn slash(&self, offline_time: u32, total_offenline: u32, param: SlashParams) -> Perbill;
    fn spid(&self) -> GeodeIdd<Offender>;
}

#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub struct GeodeOffence<Offender> {
    pub offender: Offender,
    pub geoid: GeodeIdd<Offender>,
}

#[derive(RuntimeDebug, Default, Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Encode, Decode)]
pub struct GeodeIdd<AccountId> {
    pub offline_time: u64,
    pub start_time: u64,
    pub provider: AccountId,
    pub user: AccountId,
}

impl<Offender: Clone> AutomataOffence<Offender> for GeodeOffence<Offender> {
    const ID: Kind = *b"geode1234:offlin";

    type SpecialId = u64;

    fn kind(&self) -> Kind {
        Self::ID
    }

    fn offender(&self) -> Offender {
        self.offender.clone()
    }

    fn slash(&self, offline_time: u32, _total_offenline: u32, _param: SlashParams) -> Perbill {
        Perbill::from_parts(5 * offline_time)
    }

    fn spid(&self) -> GeodeIdd<Offender> {
        self.geoid.clone()
    }
}

/// Errors that may happen on offence reports.
#[derive(PartialEq, sp_runtime::RuntimeDebug)]
pub enum AutomataOffenceError {
    DuplicateReport,
    NotReportSpan,
    Other(u8),
}

impl sp_runtime::traits::Printable for AutomataOffenceError {
    fn print(&self) {
        "OffenceError".print();
        match self {
            Self::DuplicateReport => "DuplicateReport".print(),
            Self::NotReportSpan => "NotReportSpan".print(),
            Self::Other(e) => {
                "Other".print();
                e.print();
            }
        }
    }
}

/// A trait for decoupling offence reporters from the actual handling of offence reports.
pub trait AutomataReportOffence<Reporter, Offender, O: AutomataOffence<Offender>> {
    fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), AutomataOffenceError>;
    fn is_known_offence(offenders: &[Offender]) -> bool;
}

/// handle all offence events in this module
pub trait OnAutomataOffenceHandler<Reporter, Block, Res> {
    fn on_offence(
        offenders: AutomataOffenceDetails<Reporter, Block>,
        slash_fraction: Perbill,
    ) -> Result<Res, ()>;

    fn can_report() -> bool;
}

/// storage for Automata Offence Details
#[derive(Clone, PartialEq, Eq, Default, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct AutomataOffenceDetails<Who, Block> {
    pub offender: Who,
    pub reporters: Vec<Who>,
    pub blocknum: Block,
    pub reason: OffenceReason,
    pub provider: Who,
    pub user: Who,
    pub offline_count: u32,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
pub enum OffenceReason {
    PermanentOffline,
    Offline,
    AttestError,
    Unkown,
}

impl Default for OffenceReason {
    fn default() -> Self {
        OffenceReason::PermanentOffline
    }
}

/// All offline records of a Geode
#[derive(Clone, Default, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct OffenceTempRecord<ID> {
    pub geodeid: ID,
    pub current_offline: u32,
    pub historical_offline: u32,
}

impl<ID> OffenceTempRecord<ID> {
    pub fn new(id: ID) -> Self {
        Self {
            geodeid: id,
            current_offline: 1,
            historical_offline: 1,
        }
    }

    pub fn increase(&mut self) {
        self.current_offline += 1;
        self.historical_offline += 1;
    }
}

///  Extra Slash :
///  0 to first_domain : base_slash_rate
///  first_domain to second_domain : second_slash_rate
///  second_domain to third_domain : third_slash_rate
#[derive(Clone, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct SlashParams {
    pub time_span: u64,
    pub base_slash_rate: Perbill,
    pub first_domain: u64,
    pub second_slash_rate: Perbill,
    pub second_domain: u64,
    pub third_slash_rate: Perbill,
    pub third_domain: u64,
}

impl Default for SlashParams {
    fn default() -> Self {
        SlashParams {
            time_span: 100,
            base_slash_rate: Perbill::from_percent(5),
            first_domain: 3,
            second_slash_rate: Perbill::from_percent(10),
            second_domain: 6,
            third_slash_rate: Perbill::from_percent(15),
            third_domain: 10,
        }
    }
}
