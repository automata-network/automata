// SPDX-License-Identifier: Apache-2.0

use super::Config;
use codec::{Decode, Encode};
pub use pallet_stake::{AutomataOffence, AutomataReportOffence, GeodeIdd, Kind, SlashParams};
use sp_runtime::{Perbill, RuntimeDebug};
use sp_std::prelude::*;
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub struct GeodeOffence<Offender> {
    pub offender: Offender,
    pub geoid: GeodeIdd<Offender>,
}

pub trait GeodeOffenceTrait<Offender>: AutomataOffence<Offender> {
    fn new(account: Offender, blocknum: u64, provider: Offender, user: Offender) -> Self;
}
impl<Offender: Clone> GeodeOffenceTrait<Offender> for GeodeOffence<Offender> {
    fn new(account: Offender, blocknum: u64, provider: Offender, user: Offender) -> Self {
        GeodeOffence {
            offender: account,
            geoid: GeodeIdd::<Offender> {
                offline_time: blocknum,
                start_time: 0,
                provider,
                user,
            },
        }
    }
}
#[derive(RuntimeDebug, Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Encode, Decode)]
pub struct GeodeInfo {
    pub orderid: u64,
    pub dealid: u64,
}

impl<Offender: Clone> AutomataOffence<Offender> for GeodeOffence<Offender> {
    const ID: Kind = *b"geode1234:offlin";

    type SpecialId = GeodeInfo;

    fn kind(&self) -> Kind {
        Self::ID
    }

    fn offender(&self) -> Offender {
        self.offender.clone()
    }

    fn slash(&self, offline_time: u32, _total_offline: u32, _param: SlashParams) -> Perbill {
        //TODO:: define the slash rule here
        Perbill::from_parts(1 * offline_time)
    }

    fn spid(&self) -> GeodeIdd<Offender> {
        self.geoid.clone()
    }
}

pub trait GeodeReport<T: Config> {
    type Offence: GeodeOffenceTrait<T::AccountId>;
    fn report(reporters: Vec<T::AccountId>, offence: Self::Offence);
}

pub struct GeodeReportOffenceHandler<I, R, O = GeodeOffence<I>> {
    _phantom: sp_std::marker::PhantomData<(I, R, O)>,
}

impl<T, R, O> GeodeReport<T> for GeodeReportOffenceHandler<T::AccountId, R, O>
where
    T: Config,
    R: AutomataReportOffence<T::AccountId, T::AccountId, O>,
    O: GeodeOffenceTrait<T::AccountId>,
{
    type Offence = O;

    fn report(reporters: Vec<T::AccountId>, offence: Self::Offence) {
        let _res = R::report_offence(reporters, offence);
    }
}
