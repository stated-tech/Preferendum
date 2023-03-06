#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{*, DispatchResult},
		sp_runtime::traits::IntegerSquareRoot,
		traits::{Currency, LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type Currency: Currency<Self::AccountId, Balance = u128>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		type MaxVotes: Get<u32>; // We added this (aswell in the mock and the lib.rs of the runtime) to create a maximum of
						 // number of our BoundedVec.
		type MaxSubjects: Get<u32>;	// Number max of subjects by proposal	 
	}

	#[pallet::storage]
	#[pallet::getter(fn registered_voters)]
	pub type RegisteredVoters<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>; // A list of all voters registered by the root

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	// pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, <T as
	// frame_system::Config>::Hash, Proposal<T>>;
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, u32, Proposal<T>>; // Map the id with the proposal

	#[pallet::storage]
	#[pallet::getter(fn aye_vote)]
	pub type NumberAyebyId<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32>; // Number of Aye by id

	#[pallet::storage]
	#[pallet::getter(fn nay_vote)]
	pub type NumberNaybyId<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32>; // Number of  Nay by id

	#[pallet::storage]
	#[pallet::getter(fn id_proposal)]
	pub type IdProposal<T> = StorageValue<_, u32>; // Each time a proposal is created, it increases by one
											   //pub type IdProposal<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn numer_quorum)]
	pub type NumberQuorumforNewVoter<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32>; // Number of people that have accepted the new participant									   
	
	#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Clone)] // Two possibilities of the vote (accept or reject)
	pub enum Vote {
		Aye(u32),
		Nay(u32),
	}

	#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Clone)] // If the proposal is in progress or finished
	pub enum ProposalStatus {
		Inprogress,
		Finished,
	}

	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Subject<T: Config> { // One subject of a proposal
		pub text: T::Hash,
	}

	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposition<T: Config> { // One proposition of one subject of a proposal
		pub text: T::Hash,
	}


	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		// Our proposal with :
		// Orelse <Hash>
		pub text: T::Hash,                              // The text hashed of the proposal
		pub id: u32,                                    // Its id
		pub progress: ProposalStatus,                   // Wether it is in progress or finished
		pub who: BoundedVec<T::AccountId, T::MaxVotes>, // A vector of all people that have voted
		pub amount: BoundedVec<u32, T::MaxVotes>,       /* A vector of the amount voted (same
		                                                 * order as people above) */
		pub subjects : BoundedVec<Subject<T>, T::MaxSubjects>,
		pub propositions : BoundedVec<BoundedVec<Proposition<T>, T::MaxSubjects>, T::MaxSubjects>,
	}



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewVoter {
			// When root has registered someone new
			who: T::AccountId,
		},
		NewProposal {
			// When a new proposal is created
			text: T::Hash,
			author: T::AccountId,
			id: u32,
		},
		NewVote {
			// When someone has voted
			vote: Vote,
			who: T::AccountId,
			id: u32,
		},
		ProposalFinished {
			// When the proposal is finished
			id: u32,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotRegistered,    // A user not registered who try to vote or create a proposal
		ProposalNotExist, // Vote for a proposal that does not exist
		ProposalFinished, // Vote for a proposal that has finished
		NotEnoughBalance, // Not enough balance to vote for the amount asked
		UserEverVoted,    // If you try to vote morre than once
		OutBounded,       //If too many value are in the vectors
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		// #[pallet::call_index(8)] 
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		// pub fn quorum_voters(origin: OriginFor<T> , id : u32) -> DispatchResult { // Id of the proposal
		// 	let who = ensure_signed(origin.clone())?;
		// 	ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered
		// 	let nb = <NumberQuorumforNewVoter<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)? ; 
		// 	let nb_register = RegisteredVoters::<T>::len();
		// 	Ok(())
		// }
		
		#[pallet::call_index(7)] // ATTENTION RAJOUTER ETRE SUR QUE A JAMAIS VOTE
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn quorum_add_voters(origin: OriginFor<T> , id : u32) -> DispatchResult { // Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered
			let nb = <NumberQuorumforNewVoter<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)? ; 
			NumberQuorumforNewVoter::<T>::insert(id, nb+1);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2).ref_time())]
		pub fn register_voter(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			// Let's register voters
			ensure_root(origin)?; // Only root can do this
					  // T::RegistrationOrigin::ensure_origin(origin)?;
			RegisteredVoters::<T>::insert(&who, ()); // He inserts "who"
			Self::deposit_event(Event::NewVoter { who });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2).ref_time())]
		pub fn create_proposal(origin: OriginFor<T>, text: T::Hash) -> DispatchResult {
			let author = ensure_signed(origin)?; // Make sure it is signed
			ensure!(RegisteredVoters::<T>::contains_key(author.clone()), <Error<T>>::NotRegistered); // Assure that the person is registered

			let id: u32; // Id of the proposal

			match <IdProposal<T>>::get() {
				// Return an error if the value has not been set.
				None => {
					<IdProposal<T>>::put(1);
					id = 1; // If it is the first time, we put it to one
				},
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					// Update the value in storage with the incremented result.
					<IdProposal<T>>::put(old + 1); // If not the first proposal, we increment by one
					id = old + 1;
				},
			}

			//<IdProposal<T>>::mutate(|x| {*x+=1}); // We could use that with valuequery aswell.

			let proposal: Proposal<T> = Proposal {
				// we create the proposal
				text: text.clone(),
				id: id.clone(),
				progress: ProposalStatus::Inprogress,
				who: Default::default(), // Default meen vector empty
				amount: Default::default(),
				subjects : Default::default(),
				propositions : Default::default(),
			};
			<Proposals<T>>::insert(id, proposal);
			Self::deposit_event(Event::NewProposal { text, author, id });

			Ok(())
		}


		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_subject(origin: OriginFor<T>, id: u32 , text: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered
	
			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			proposal
			.subjects
			.try_push(Subject{text : text.clone()})
			.map_err(|_x| <Error<T>>::OutBounded)?;
			<Proposals<T>>::insert(id, proposal); // We update the proposal
			// Self::deposit_event(Event::ProposalFinished { id }); // rajouter bon event
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn stop_proposal(origin: OriginFor<T>, id: u32) -> DispatchResult {
			// We make the proposal stoped and token back to the user
			ensure_root(origin)?; // Only root can do that

			let proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			let proposal_finished: Proposal<T> = Proposal {
				text: proposal.text,
				id: proposal.id,
				progress: ProposalStatus::Finished, // we have updated the progress
				who: proposal.who,
				amount: proposal.amount,
				subjects : Default::default(),
				propositions: Default::default(),
			};
			let prop = &proposal_finished;
			<Proposals<T>>::remove(&proposal.id); // We remove the proposal with the idea
			<Proposals<T>>::insert(&proposal_finished.id, prop); // We put the new one inside

			let current_proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?;

			for i in 0..current_proposal.who.len() {
				// For each voter
				let x = current_proposal.amount[i];
				T::Currency::unreserve(&current_proposal.who[i], (x * x).into()); // We unreserve
				                                                  // the amount
				                                                  // reserved
			}

			Self::deposit_event(Event::ProposalFinished { id });
			Ok(())
		}

	

	







		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5,3).ref_time())]
		pub fn create_vote(origin: OriginFor<T>, id: u32, vote: Vote) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.binary_search(&who).err() != None, <Error<T>>::UserEverVoted); // Make sure the user has not voted
			ensure!(proposal.progress != ProposalStatus::Finished, <Error<T>>::ProposalFinished); //Make sure the proposal is in progress

			match vote {
				Vote::Aye(x) => {
					//ensure!(T::Currency::free_balance(&who)>=x.into(),
					// <Error<T>>::NotEnoughBalance); // we could have done that aswell

					T::Currency::reserve(&who, (x * x).into())?; // If not enough balance, will return Insufficient balance

					match <NumberAyebyId<T>>::get(id) {
						// Get the last number of vote
						// Return an error if the value has not been set.
						None => {
							// If none, mean we are the first vote
							<NumberAyebyId<T>>::insert(id, x); // We put the value in
							let mut new_proposal =
								<Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?;
							new_proposal
								.who
								.try_push(who.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							new_proposal
								.amount
								.try_push(x.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							<Proposals<T>>::insert(id, new_proposal); // We update the proposal by putting the new vote (who and
							              // amount)
						},
						Some(old) => {
							// If  there is a value, we add it

							<NumberAyebyId<T>>::insert(
								id,
								x + old, // We add it
							);
							let mut new_proposal =
								<Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?;
							new_proposal
								.who
								.try_push(who.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							new_proposal
								.amount
								.try_push(x.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							<Proposals<T>>::insert(id, new_proposal); // We update the proposal with who-amount
						},
					}
				},
				Vote::Nay(y) => {
					// Exactly the same that Vote::Aye

					match <NumberNaybyId<T>>::get(id) {
						None => {
							<NumberNaybyId<T>>::insert(id, IntegerSquareRoot::integer_sqrt(&y)); // We add the value
							let mut new_proposal =
								<Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?;
							new_proposal
								.who
								.try_push(who.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							new_proposal
								.amount
								.try_push(y.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							<Proposals<T>>::insert(id, new_proposal); // We update the proposal
						},
						Some(old) => {
							// Increment the value read from storage; will error in the event of
							// overflow.
							// Update the value in storage with the incremented result.
							<NumberNaybyId<T>>::insert(
								id,
								IntegerSquareRoot::integer_sqrt(&y) + old, /* We add the value
								                                            * to the old */
							);
							let mut new_proposal =
								<Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?;
							new_proposal
								.who
								.try_push(who.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							new_proposal
								.amount
								.try_push(y.clone())
								.map_err(|_x| <Error<T>>::OutBounded)?;
							<Proposals<T>>::insert(id, new_proposal);
						},
					}
				},
			}
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::NewVote { vote, who, id });

			Ok(())
		}

		
	}
}



