// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Runtime parameters.

use std::fmt::Display;

use sc_chain_spec::ChainSpec;
use scale_info::{form::PortableForm, TypeDef, TypeDefPrimitive};

/// The Aura ID used by the Aura consensus
#[derive(PartialEq)]
pub enum AuraConsensusId {
	/// Ed25519
	Ed25519,
	/// Sr25519
	Sr25519,
}

/// The choice of consensus for the parachain omni-node.
#[derive(PartialEq)]
pub enum Consensus {
	/// Aura consensus.
	Aura(AuraConsensusId),
}

/// The choice of block number for the parachain omni-node.
#[derive(PartialEq)]
pub enum BlockNumber {
	/// u32
	U32,
	/// u64
	U64,
}

impl Display for BlockNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BlockNumber::U32 => write!(f, "u32"),
			BlockNumber::U64 => write!(f, "u64"),
		}
	}
}

impl Into<TypeDefPrimitive> for BlockNumber {
	fn into(self) -> TypeDefPrimitive {
		match self {
			BlockNumber::U32 => TypeDefPrimitive::U32,
			BlockNumber::U64 => TypeDefPrimitive::U64,
		}
	}
}

impl BlockNumber {
	fn from_type_def(type_def: &TypeDef<PortableForm>) -> Option<BlockNumber> {
		match type_def {
			TypeDef::Primitive(TypeDefPrimitive::U32) => Some(BlockNumber::U32),
			TypeDef::Primitive(TypeDefPrimitive::U64) => Some(BlockNumber::U64),
			_ => None,
		}
	}
}

/// Helper enum listing the supported Runtime types
#[derive(PartialEq)]
pub enum Runtime {
	/// None of the system-chain runtimes, rather the node will act agnostic to the runtime ie. be
	/// an omni-node, and simply run a node with the given consensus algorithm.
	Omni(BlockNumber, Consensus),
}

/// Helper trait used for extracting the Runtime variant from the chain spec ID.
pub trait RuntimeResolver {
	/// Extract the Runtime variant from the chain spec ID.
	fn runtime(&self, chain_spec: &dyn ChainSpec) -> sc_cli::Result<Runtime>;
}

/// Default implementation for `RuntimeResolver` that just returns
/// `Runtime::Omni(BlockNumber::U32, Consensus::Aura(AuraConsensusId::Sr25519))`.
pub struct DefaultRuntimeResolver;

impl RuntimeResolver for DefaultRuntimeResolver {
	fn runtime(&self, _chain_spec: &dyn ChainSpec) -> sc_cli::Result<Runtime> {
		Ok(Runtime::Omni(BlockNumber::U32, Consensus::Aura(AuraConsensusId::Sr25519)))
	}
}

/// Logic that inspects runtime's metadata for Omni Node compatibility.
pub mod metadata {
	use super::BlockNumber;
	use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};

	/// Checks if pallet exists in runtime's metadata based on pallet name.
	pub fn pallet_exists(
		metadata: &RuntimeMetadataPrefixed,
		name: &str,
	) -> Result<bool, sc_service::error::Error> {
		match &metadata.1 {
			RuntimeMetadata::V14(inner) => Ok(inner.pallets.iter().any(|p| p.name == name)),
			RuntimeMetadata::V15(inner) => Ok(inner.pallets.iter().any(|p| p.name == name)),
			_ => Err(sc_service::error::Error::Application(
				anyhow::anyhow!(format!(
					"Metadata version {} not supported for checking against pallet existence.",
					metadata.0
				))
				.into(),
			)),
		}
	}

	/// Get the configured runtime's block number type from `frame-system` pallet storage.
	pub fn runtime_block_number(
		metadata: &RuntimeMetadataPrefixed,
	) -> Result<BlockNumber, sc_service::error::Error> {
		// Macro to define reusable logic for processing metadata.
		macro_rules! process_metadata {
			($metadata:expr) => {{
				$metadata
					.pallets
					.iter()
					.find(|p| p.name == "System")
					.and_then(|system| system.storage.as_ref())
					.and_then(|storage| storage.entries.iter().find(|entry| entry.name == "Number"))
					.and_then(|number_ty| match number_ty.ty {
						frame_metadata::v14::StorageEntryType::Plain(ty) => Some(ty.id),
						_ => None,
					})
					.and_then(|number_id| $metadata.types.resolve(number_id))
					.and_then(|portable_type| BlockNumber::from_type_def(&portable_type.type_def))
			}};
		}

		let err_msg = "Can not get block number type from `frame-system-pallet` storage.";
		match &metadata.1 {
			RuntimeMetadata::V14(meta) => process_metadata!(meta).ok_or(sc_service::error::Error::Application(
					anyhow::anyhow!(err_msg).into())),
			RuntimeMetadata::V15(meta) => process_metadata!(meta).ok_or(sc_service::error::Error::Application(
					anyhow::anyhow!(err_msg).into())),
			_ =>
				Err(sc_service::error::Error::Application(
					anyhow::anyhow!(format!(
						"Metadata version {} not supported for checking block number type stored in `frame-system-pallet` storage.",
						metadata.0
					))
					.into(),
				)),
		}
	}
}
