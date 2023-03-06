#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
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
		type MaxSubjects: Get<u32>; // Number max of subjects by proposal
		type MaxAlternatives: Get<u32>; // Number max of subjects by proposal
	}

	#[pallet::storage]
	#[pallet::getter(fn registered_voters)]
	pub type RegisteredVoters<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>; // A list of all voters registered by the root

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, u32, Proposal<T>>; // Map the id with the proposal

	#[pallet::storage]
	#[pallet::getter(fn id_proposal)]
	pub type IdProposal<T> = StorageValue<_, u32>; // Each time a proposal is created, it increases by one
											   //pub type IdProposal<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn numer_quorum)]
	pub type StatusAlternativeNewKick<T: Config> =
		StorageMap<_, Blake2_128Concat, (u32, T::AccountId, u32, u32), u32>; // Status of the if an alternative is kicked by user

	#[pallet::storage]
	#[pallet::getter(fn numer_quorum_total)]
	pub type NumberQuorumforNewKickedAlt<T: Config> =
		StorageMap<_, Blake2_128Concat, (u32, u32, u32), u32>; // Number of people that have accepted to kick an alternative

	#[pallet::storage]
	#[pallet::getter(fn status_vote)]
	pub type StatusToVote<T: Config> = StorageMap<_, Blake2_128Concat, (u32, T::AccountId), u32>; // Status of if we should begin the vote

	#[pallet::storage]
	#[pallet::getter(fn nb_quorum_vote)]
	pub type NumberQuorumForStatusVote<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32>; // Number of people that have accepted to begin the vote

	#[pallet::storage]
	#[pallet::getter(fn status_stop_vote)]
	pub type StatusToStopVote<T: Config> =
		StorageMap<_, Blake2_128Concat, (u32, T::AccountId), u32>; //Status of if we should stop  the vote

	#[pallet::storage]
	#[pallet::getter(fn nb_quorum_stop_vote)]
	pub type NumberQuorumForStatusStopVote<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32>; //  Number of people that have accepted to stop the vote

	#[pallet::storage]
	#[pallet::getter(fn vote_byid_byquorum)]
	pub type VoteByQuorum<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(u32, T::AccountId),
		BoundedVec<BoundedVec<u32, T::MaxSubjects>, T::MaxAlternatives>,
	>; // Vote of users by id

	#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Clone)] // If the proposal is in progress or finished
	pub enum ProposalStatus {
		Inprogress,
		Vote,
		Finished,
	}

	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Subject<T: Config> {
		// One subject of a proposal
		pub text: T::Hash,
	}

	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Alternative<T: Config> {
		// One proposition of one subject of a proposal
		pub text: T::Hash,
	}

	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		pub text: T::Hash,                              // The text hashed of the proposal
		pub id: u32,                                    // Its id
		pub percentage_quorum: u32,                     /* Percentage of people to accept a
		                                                 * decision */
		pub progress: ProposalStatus, // Wether it is in progress or finished
		pub who: BoundedVec<T::AccountId, T::MaxVotes>, // A vector of all people in this proposal
		pub subjects: BoundedVec<Subject<T>, T::MaxSubjects>, // List of subjects
		pub alternatives:
			BoundedVec<BoundedVec<Alternative<T>, T::MaxSubjects>, T::MaxAlternatives>, /* List of alternatives for each subjects */
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
		ProposalFinished {
			// When the proposal is finished
			id: u32,
		},
		NewVoterQuorum {
			who: T::AccountId,
		},
		SubjectAdd {
			id: u32,
			sub: T::Hash,
		},
		AlternativeAdd {
			id: u32,
			alt: T::Hash,
		},
		Votee {
			id: u32,
			vote: BoundedVec<BoundedVec<u32, T::MaxSubjects>, T::MaxAlternatives>,
			who: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotRegistered,     // A user not registered who try to vote or create a proposal
		ProposalNotExist,  // Vote for a proposal that does not exist
		ProposalFinished,  // Vote for a proposal that has finished
		OutBounded,        //If too many value are in the vectors
		NotInTheQuorum,    // If a user is not in the proposal
		Problem,           // Problem (help in test part)
		VoteFailed,        /* Fail if the number of alternatives does not match to what gave
		                    * the user */
		NotTimeVote,       // The status of the proposal is not Vote
		EverInTheProposal, // Already in the proposal
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			// Let's create the proposal
			let mut proposal: Proposal<T> = Proposal {
				// we create the proposal
				text: text.clone(),
				id: id.clone(),
				progress: ProposalStatus::Inprogress,
				percentage_quorum: 50,
				who: Default::default(), // Default meen vector empty
				subjects: Default::default(),
				alternatives: Default::default(),
			};
			proposal
				.who
				.try_push(author.clone()) // We push by default the author in the list of person in the proposal
				.map_err(|_x| <Error<T>>::OutBounded)?;

			// We add it in the storage
			<Proposals<T>>::insert(id, proposal);
			Self::deposit_event(Event::NewProposal { text, author, id });

			Ok(())
		}

		#[pallet::call_index(3)] // ATTENTION RAJOUTER ETRE SUR QUE A JAMAIS VOTE
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_voters(origin: OriginFor<T>, id: u32) -> DispatchResult {
			// Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::Problem)?; //Get the proposal
			ensure!(!proposal.who.contains(&who), <Error<T>>::EverInTheProposal); // Make sure the person is not already in the proposal

			proposal.who.try_push(who.clone()).map_err(|_x| <Error<T>>::OutBounded)?;
			<Proposals<T>>::insert(id, proposal); // We update the proposal

			Self::deposit_event(Event::NewVoterQuorum { who });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_subject(origin: OriginFor<T>, id: u32, text: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal
			ensure!(proposal.progress == ProposalStatus::Inprogress, <Error<T>>::NotTimeVote); // Make sure we are in the good status

			proposal // We add the subject
				.subjects
				.try_push(Subject { text: text.clone() })
				.map_err(|_x| <Error<T>>::OutBounded)?;

			proposal // We put nothing in the alternatives, we just initialize it
				.alternatives
				.try_push(Default::default())
				.map_err(|_x| <Error<T>>::OutBounded)?;

			Self::deposit_event(Event::SubjectAdd { id: proposal.id, sub: text }); // rajouter bon event
			<Proposals<T>>::insert(id, proposal); // We update the proposal
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_alternatives(
			origin: OriginFor<T>,
			id: u32,
			num_sub: u32,
			text: T::Hash,
		) -> DispatchResult {
			// num_sub begin to 0
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal
			ensure!(proposal.subjects.len() > num_sub as usize, <Error<T>>::Problem); // Make sure the subject exists MODIFY ERROR NAME
			ensure!(proposal.progress == ProposalStatus::Inprogress, <Error<T>>::NotTimeVote); // Make sure good progress

			proposal // Add the alternative
				.alternatives[num_sub as usize]
				.try_push(Alternative { text: text.clone() })
				.map_err(|_x| <Error<T>>::OutBounded)?;

			Self::deposit_event(Event::AlternativeAdd { id: proposal.id, alt: text }); // rajouter bon event
			<Proposals<T>>::insert(id, proposal); // We update the proposal

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn quorum_kick_proposal(
			origin: OriginFor<T>,
			id: u32,
			sub: u32,
			alt: u32,
		) -> DispatchResult {
			// Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal

			let status_voted = <StatusAlternativeNewKick<T>>::get((id, who.clone(), sub, alt)); // We get the option to check if has alrezady voted

			match status_voted {
				Some(_) => {}, /* If we have something, it means the person has already voted,
				                 * so we we do nothing */
				None => {
					// Orelse, we add a value, so that he won't be able to vote twice
					StatusAlternativeNewKick::<T>::insert((id, who.clone(), sub, alt), 1); // We add a value so that it shows he has voted

					let nb = <NumberQuorumforNewKickedAlt<T>>::get((id, sub, alt)); // Get the number of vote
					match nb {
						Some(x) => {
							// If we have vote, we add one below
							NumberQuorumforNewKickedAlt::<T>::insert((id, sub, alt), x + 1);
							if (x as usize + 1) * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								// If quorum verified,
								proposal.alternatives[sub as usize].remove(alt as usize); // Remove it
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
						None => {
							// If no vote, then we update the number with one below
							NumberQuorumforNewKickedAlt::<T>::insert((id, sub, alt), 1);
							if 1 as usize * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								proposal.alternatives[sub as usize].remove(alt as usize);
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
					}
				},
			}
			Ok(())
		}

		#[pallet::call_index(7)] // RAJOUTER QUE DANS AUTRES FONCTIONS SI A ETE KICKE PAS LE DROIT D EJOIN
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn quorum_propose_vote(origin: OriginFor<T>, id: u32) -> DispatchResult {
			// Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal

			let status_voted = <StatusToVote<T>>::get((id, who.clone())); // We get the option of if a user has voted

			match status_voted {
				Some(_) => {}, /* If we have something, it means the person has already voted,
				                 * so we we do nothing */
				None => {
					// Orelse, we add a value, so that he won't be able to vote twice
					StatusToVote::<T>::insert((id, who.clone()), 1); //// We add a value so that it shows he has voted

					let nb = <NumberQuorumForStatusVote<T>>::get(id);
					match nb {
						Some(x) => {
							NumberQuorumForStatusVote::<T>::insert(id, x + 1); // We increase by one
							if (x as usize + 1) * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								proposal.progress = ProposalStatus::Vote;
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
						None => {
							NumberQuorumForStatusVote::<T>::insert(id, 1); // As it is the first vote, we put the number to 1
							if 1 as usize * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								proposal.progress = ProposalStatus::Vote;
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
					}
				},
			}

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn quorum_propose_stop_vote(origin: OriginFor<T>, id: u32) -> DispatchResult {
			// Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal

			let status_voted = <StatusToStopVote<T>>::get((id, who.clone())); // Get the option of if has voted

			match status_voted {
				Some(_) => {}, /* If we have something, it means the person has already voted,
				                 * so we we do nothing */
				None => {
					// Orelse, we add a value, so that he won't be able to vote twice
					StatusToStopVote::<T>::insert((id, who.clone()), 1);

					let nb = <NumberQuorumForStatusStopVote<T>>::get(id); // Number of vote
					match nb {
						Some(x) => {
							// We increase by one below
							NumberQuorumForStatusStopVote::<T>::insert(id, x + 1);
							if (x as usize + 1) * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								proposal.progress = ProposalStatus::Finished;
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
						None => {
							// If no vote, then we put below the value to 1
							NumberQuorumForStatusStopVote::<T>::insert(id, 1);
							if 1 as usize * 100 >
								proposal.who.len() * proposal.percentage_quorum as usize + 1
							{
								proposal.progress = ProposalStatus::Finished;
								<Proposals<T>>::insert(id, proposal); // We update the proposal
							}
						},
					}
				},
			}

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn vote(
			origin: OriginFor<T>,
			id: u32,
			vote: BoundedVec<BoundedVec<u32, T::MaxSubjects>, T::MaxAlternatives>,
		) -> DispatchResult {
			// We make the proposal stoped and token back to the user
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
			ensure!(proposal.who.contains(&who), <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal
			ensure!(proposal.progress == ProposalStatus::Vote, <Error<T>>::NotTimeVote); // Make sure it is the good status

			ensure!(proposal.subjects.len() == vote.len(), <Error<T>>::VoteFailed); // Fail if the number of subjects don't match

			// We calcul the number of current alternatives and votres
			let mut nb_alt = 0;
			let mut nb_vote = 0;
			for i in 0..proposal.subjects.len() {
				nb_alt = nb_alt + proposal.alternatives[i].len();
				nb_vote = nb_vote + vote[i].len();
			}

			ensure!(nb_alt == nb_vote, <Error<T>>::VoteFailed); // Fail if the number of alternatives don't match

			let vote_storage = <VoteByQuorum<T>>::get((id, who.clone()));

			match vote_storage {
				Some(_) => (), /* Do nothing if already voted : WE WILL IMPLEMENT LATER THE
				                 * ABILITY TO CHANGE YOUR VOTE */
				None => {
					<VoteByQuorum<T>>::insert((id, who.clone()), vote.clone());
				},
			}

			Self::deposit_event(Event::Votee { id, vote, who });
			Ok(())
		}
	}
}
