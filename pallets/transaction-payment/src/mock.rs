// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use crate as pallet_transaction_payment;

use frame_support::{
	derive_impl,
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstU32, ConstU64, Imbalance, OnUnbalanced},
	weights::{Weight, WeightToFee as WeightToFeeT},
};
use frame_system as system;
use frame_system::{
	header_builder::da::HeaderExtensionBuilder, mocking::MockUncheckedExtrinsic,
	test_utils::TestRandomness,
};
use pallet_balances::Call as BalancesCall;

/// An unchecked extrinsic type to be used in tests.
type UncheckedExtrinsic = MockUncheckedExtrinsic<Runtime>;

/// An implementation of `sp_runtime::traits::Block` to be used in tests.
type Block = frame_system::mocking::MockDaBlock<Runtime>;

type BlockNumber = u32;
type Balance = u64;

frame_support::construct_runtime!(
	pub enum Runtime
	{
		System: system,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
	}
);

pub(crate) const CALL: &<Runtime as frame_system::Config>::RuntimeCall =
	&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 69 });

parameter_types! {
	pub(crate) static ExtrinsicBaseWeight: Weight = Weight::zero();
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(Weight::zero())
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = Weight::from_parts(1024, u64::MAX).into();
			})
			.build_or_panic()
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub static WeightToFee: u64 = 1;
	pub static TransactionByteFee: u64 = 1;
	pub static OperationalFeeMultiplier: u8 = 5;
}

#[derive_impl(system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type BaseCallFilter = frame_support::traits::Everything;
	type Block = Block;
	type BlockWeights = BlockWeights;
	type BlockHashCount = BlockHashCount;
	type HeaderExtensionBuilder = HeaderExtensionBuilder<Runtime>;
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type Randomness = TestRandomness<Runtime>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SubmittedDataExtractor = ();
	type UncheckedExtrinsic = UncheckedExtrinsic;
	type MaxDiffAppIdPerBlock = ConstU32<1_024>;
	type MaxTxPerAppIdPerBlock = ConstU32<8_192>;
}

impl pallet_balances::Config for Runtime {
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

impl WeightToFeeT for WeightToFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(WEIGHT_TO_FEE.with(|v| *v.borrow()))
	}
}

impl WeightToFeeT for TransactionByteFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
	}
}

parameter_types! {
	pub(crate) static TipUnbalancedAmount: u64 = 0;
	pub(crate) static FeeUnbalancedAmount: u64 = 0;
}

pub struct DealWithFees;
impl OnUnbalanced<pallet_balances::NegativeImbalance<Runtime>> for DealWithFees {
	fn on_unbalanceds<B>(
		mut fees_then_tips: impl Iterator<Item = pallet_balances::NegativeImbalance<Runtime>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			FeeUnbalancedAmount::mutate(|a| *a += fees.peek());
			if let Some(tips) = fees_then_tips.next() {
				TipUnbalancedAmount::mutate(|a| *a += tips.peek());
			}
		}
	}
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
	type LengthToFee = TransactionByteFee;
	type FeeMultiplierUpdate = ();
	type LengthMultiplierUpdate = ();
}
