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
	#[pallet::getter(fn id_proposal)]
	pub type IdProposal<T> = StorageValue<_, u32>; // Each time a proposal is created, it increases by one
											   //pub type IdProposal<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
	#[pallet::getter(fn numer_quorum)]
	pub type StatusVotedNewVoter<T: Config> = StorageMap<_, Blake2_128Concat, (u32,T::AccountId,T::AccountId), u32>; // Number of people that have accepted the new participant			
    
	#[pallet::storage]
	#[pallet::getter(fn numer_quorum_total)]
	pub type NumberQuorumforNewVoter<T: Config> = StorageMap<_, Blake2_128Concat, (u32,T::AccountId), u32>; // Number of people that have accepted the new participant		

    #[pallet::storage]
	#[pallet::getter(fn vote_byid_byquorum)]
	pub type VoteByQuorum<T: Config> = StorageMap<_, Blake2_128Concat, (u32,T::AccountId), BoundedVec<BoundedVec<u32, T::MaxVotes>, T::MaxVotes>> ; // Number of people that have accepted the new participant								   

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
	pub struct Alternative<T: Config> { // One proposition of one subject of a proposal
		pub text: T::Hash,
	}


	#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		// Our proposal with :
		// Orelse <Hash>
		pub text: T::Hash,                              // The text hashed of the proposal
		pub id: u32,                                    // Its id
        pub percentage_quorum : u32,
		pub progress: ProposalStatus,                   // Wether it is in progress or finished
		pub who: BoundedVec<T::AccountId, T::MaxVotes>, // A vector of all people in this proposal
		pub subjects : BoundedVec<Subject<T>, T::MaxSubjects>,
		pub alternatives : BoundedVec<BoundedVec<Alternative<T>, T::MaxSubjects>, T::MaxSubjects>,
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
        EverInTheProposal, // When someone in a proposal try to be added again
        NotInTheQuorum,
        Problem,
        VoteFailed,
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

			let mut proposal: Proposal<T> = Proposal {
				// we create the proposal
				text: text.clone(),
				id: id.clone(),
				progress: ProposalStatus::Inprogress,
                percentage_quorum : 50,
				who: Default::default(), // Default meen vector empty
				subjects : Default::default(),
				alternatives : Default::default(),
			};

            proposal
            .who
            .try_push(author.clone())
            .map_err(|_x| <Error<T>>::OutBounded)?;

			<Proposals<T>>::insert(id, proposal);

			Self::deposit_event(Event::NewProposal { text, author, id });

			Ok(())
		}


		#[pallet::call_index(7)] // ATTENTION RAJOUTER ETRE SUR QUE A JAMAIS VOTE
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn quorum_add_voters(origin: OriginFor<T> , id : u32 , new: T::AccountId) -> DispatchResult { // Id of the proposal
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

            let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal            
            ensure!(proposal.who.contains(&who) , <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal
            ensure!(!proposal.who.contains(&new) , <Error<T>>::Problem);

			let status_voted = <StatusVotedNewVoter<T>>::get((id,who.clone(),new.clone())); 

            match status_voted { 
                Some(_) => {}, // If we have something, it means the person has already voted, so we we do nothing
                None => { // Orelse, we add a value, so that he won't be able to vote twice
                StatusVotedNewVoter::<T>::insert((id,who.clone(),new.clone()), 1) ;
                    
                    let nb = <NumberQuorumforNewVoter<T>>::get((id,new.clone())); 
                    match nb {
                        Some(x) => {
                            NumberQuorumforNewVoter::<T>::insert((id,new.clone()), x+1);
                            if (x as usize +1) *100 > proposal.who.len() * proposal.percentage_quorum as usize +1{
                                proposal
                                .who
                                .try_push(new.clone())
                                .map_err(|_x| <Error<T>>::OutBounded)?;
                                <Proposals<T>>::insert(id, proposal); // We update the proposal
                            }
                        },
                        None => {
                            NumberQuorumforNewVoter::<T>::insert((id,new.clone()), 1);
                            if 1 as usize *100 > proposal.who.len() * proposal.percentage_quorum as usize +1{
                                proposal
                                .who
                                .try_push(new.clone())
                                .map_err(|_x| <Error<T>>::OutBounded)?;
                                <Proposals<T>>::insert(id, proposal); // We update the proposal
                            }
                        },
                    }

                },
            }

			Ok(())
		}

        
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_subject(origin: OriginFor<T>, id: u32 ,  text: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered
            
            let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
            ensure!(proposal.who.contains(&who) , <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal

			proposal
			.subjects
			.try_push(Subject{text : text.clone()})
			.map_err(|_x| <Error<T>>::OutBounded)?;

            proposal
            .alternatives
            .try_push(Default::default())           
            .map_err(|_x| <Error<T>>::OutBounded)?;
			<Proposals<T>>::insert(id, proposal); // We update the proposal
			// Self::deposit_event(Event::ProposalFinished { id }); // rajouter bon event
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn add_alternatives(origin: OriginFor<T>, id: u32 , num_sub : u32, text: T::Hash) -> DispatchResult { // num_sub begin to 0
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered
            
            let mut proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
            ensure!(proposal.who.contains(&who) , <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal
            ensure!(proposal.subjects.len()>num_sub as usize, <Error<T>>::NotInTheQuorum); // Make sure the subject exists MODIFY ERROR NAME


			proposal
			.alternatives[num_sub as usize]
			.try_push(Alternative{text : text.clone()})
			.map_err(|_x| <Error<T>>::OutBounded)?;
			<Proposals<T>>::insert(id, proposal); // We update the proposal
			Ok(())
		}

        #[pallet::call_index(20)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1).ref_time())]
		pub fn vote(origin: OriginFor<T>, id: u32 , vote : BoundedVec<BoundedVec<u32, T::MaxVotes>, T::MaxVotes> ) -> DispatchResult {
			// We make the proposal stoped and token back to the user
			let who = ensure_signed(origin.clone())?;
			ensure!(RegisteredVoters::<T>::contains_key(&who), <Error<T>>::NotRegistered); // Make sure registered

			let proposal = <Proposals<T>>::get(id).ok_or(<Error<T>>::ProposalNotExist)?; //Get the proposal
            ensure!(proposal.who.contains(&who) , <Error<T>>::NotInTheQuorum); // Make sure you are in the proposal

            ensure!(proposal.subjects.len() == vote.len() , <Error<T>>::VoteFailed);

            let mut nb_alt =0;
            let mut nb_vote = 0;
            for i in 0..proposal.subjects.len() {
                nb_alt = nb_alt + proposal.alternatives[i].len();
                nb_vote = nb_vote + vote[i].len();
            }

            ensure!(nb_alt == nb_vote , <Error<T>>::VoteFailed);

            let vote_storage = <VoteByQuorum<T>>::get((id,who.clone())); 

            match vote_storage {
                Some(_) => (), // Do nothing if already voted : WE WILL IMPLEMENT LATER THE ABILITY TO CHANGE YOUR VOTE
                None => {
                    <VoteByQuorum<T>>::insert((id,who.clone()), vote); 
                },
            }

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
                percentage_quorum: proposal.percentage_quorum,
				who: proposal.who,
				subjects : proposal.subjects,
				alternatives: proposal.alternatives,
			};
			let prop = &proposal_finished;
			<Proposals<T>>::remove(&proposal.id); // We remove the proposal with the idea
			<Proposals<T>>::insert(&proposal_finished.id, prop); // We put the new one inside

			Self::deposit_event(Event::ProposalFinished { id });
			Ok(())
		}
		
	}
}



