#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec; 

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber), ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// 当发生重要更改时，托盘使用事件通知用户。
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 事件文档应以一个数组结尾，该数组为事件参数提供描述性名称. [something, who]。
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// 当新的证明被添加到区块链时
		/// Event emitted when a proof has been claimed. [who, claim]
		ClaimCreated(T::AccountId, Vec<u8>),
		/// 当证明被移除时
		/// Event emitted when a claim is revoked by the owner. [who, claim]
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	// 错误通知用户出现问题
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		/// 错误名称应该是描述性的。
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		/// 错误应该有与之相关的有用文档。
		StorageOverflow,

		/// 证据已经被声明, 不能再次声明
		/// The proof has already been claimed.
		ProofAlreadyClaimed,
		/// 该证明不存在，因此无法撤销
		/// The proof does not exist, so it cannot be revoked.
		NoSuchProof,
		/// 该证明已被另一个帐户认领，因此调用者无法撤消它。
		/// The proof is claimed by another account, so caller can't revoke it.
		NotProofOwner,
	}


	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// Dispatchable 功能允许用户与托盘交互并调用状态更改。
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// 这些功能体现为“外在功能”，通常与交易一样。
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	// Dispatchable 函数必须使用权重进行注释，并且必须返回 DispatchResult
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		/// 一个示例 dispatchable，它接受一个单值作为参数，将值写入 storage 并发出一个事件。此函数必须由已签名的外部函数调度。
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::weight(1_000)]
		pub fn create_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {

			// 检查外部是否已签名并获取签名者, 如果外部未签名，此函数将返回错误。
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let sender = ensure_signed(origin)?;
			
			// 验证指定的证明尚未声明。
			// Verify that the specified proof has not already been claimed.         
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

			// 从 FRAME 系统模块获取块编号
			// Get the block number from the FRAME System module.
			let current_block = <frame_system::Pallet<T>>::block_number();

			// 将证明与发送者和区块号一起存储
			// Store the proof with the sender and block number.
			Proofs::<T>::insert(&proof, (&sender, current_block));

			// 发出声明已创建的事件
			// Emit an event that the claim was created.
			Self::deposit_event(Event::ClaimCreated(sender, proof));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let sender = ensure_signed(origin)?;

			// Verify that the specified proof has been claimed.
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

			// Get owner of the claim.
			let (owner, _) = Proofs::<T>::get(&proof);

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NotProofOwner);

			// Remove claim from storage.
			Proofs::<T>::remove(&proof);

			// Emit an event that the claim was erased.
			Self::deposit_event(Event::ClaimRevoked(sender, proof));

			Ok(().into())
		}
	}
}
