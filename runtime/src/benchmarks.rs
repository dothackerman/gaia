// SPDX-License-Identifier: GPL-3.0-or-later
//
// GAIA carries the repository-wide GPL-3.0-or-later licensing policy.

frame_benchmarking::define_benchmarks!(
	[frame_benchmarking, BaselineBench::<Runtime>]
	[frame_system, SystemBench::<Runtime>]
	[frame_system_extensions, SystemExtensionsBench::<Runtime>]
	[pallet_balances, Balances]
	[pallet_timestamp, Timestamp]
	[pallet_sudo, Sudo]
	[pallet_template, Template]
);
