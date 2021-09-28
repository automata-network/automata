use crate::NegativeImbalance;
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
    R: pallet_balances::Config + pallet_authorship::Config,
    <R as frame_system::Config>::AccountId: From<automata_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<automata_primitives::AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        let numeric_amount = amount.peek();
        let author = <pallet_authorship::Pallet<R>>::author();
        <pallet_balances::Pallet<R>>::resolve_creating(
            &<pallet_authorship::Pallet<R>>::author(),
            amount,
        );
        <frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit(
            author,
            numeric_amount,
        ));
    }
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
    R: pallet_balances::Config + pallet_treasury::Config + pallet_authorship::Config,
    pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
    <R as frame_system::Config>::AccountId: From<automata_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<automata_primitives::AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 80% to treasury, 20% to author
            let mut split = fees.ration(80, 20);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 100% to author
                tips.merge_into(&mut split.1);
            }
            use pallet_treasury::Pallet as Treasury;
            <Treasury<R> as OnUnbalanced<_>>::on_unbalanced(split.0);
            <ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
        }
    }
}
