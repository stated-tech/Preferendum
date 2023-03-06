use crate::{mock::*, Event, Proposal, ProposalStatus , Subject , Alternative};
use frame_support::{assert_ok, BoundedVec};
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_core::bounded_vec;


#[test]
fn test_register_voter() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root create a voter "1"
		assert_eq!(Voting::registered_voters(1), Some(())); // We make sure he is inside
	});
}

#[test]
fn create_proposal_() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added a user
        assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root added a user


		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist
		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
            who: bounded_vec![1],
			subjects : Default::default(),
			alternatives: Default::default(),
		};

		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it
	});
}

#[test]
fn add_someone_quorum() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root create a voter "1"
        assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root create a voter "1"
        assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 3)); // Root create a voter "1"


		assert_eq!(Voting::registered_voters(1), Some(())); // We make sure he is inside

        let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist


        assert_ok!(Voting::quorum_add_voters(RuntimeOrigin::signed(1),1,2));

        let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
            who: bounded_vec![1,2],
			subjects : Default::default(),
			alternatives: Default::default(),
		};

		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it

        


        assert_ok!(Voting::quorum_add_voters(RuntimeOrigin::signed(1),1,3));
        assert_ok!(Voting::quorum_add_voters(RuntimeOrigin::signed(2),1,3));

        let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
            who: bounded_vec![1,2,3],
			subjects : Default::default(),
			alternatives: Default::default(),
		};
        assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it


	});
}

#[test]
fn add_subject() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // User 1 create a proposal
		System::assert_last_event(Event::NewProposal { text: hash_text, author: 1, id: 1 }.into()); // We verify the event


		let sub = "sub1";
		let sub_hash = BlakeTwo256::hash_of(&sub);
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
			who: bounded_vec![1],
			subjects : bounded_vec![Subject{text :sub_hash}],
			alternatives : bounded_vec![Default::default()],
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it


	});
}

#[test]
fn add_alternatives() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // User 1 create a proposal
		System::assert_last_event(Event::NewProposal { text: hash_text, author: 1, id: 1 }.into()); // We verify the event


		let sub = "sub1";
		let sub_hash = BlakeTwo256::hash_of(&sub);
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash));

        let prop = "prop1";
        let prop_hash = BlakeTwo256::hash_of(&prop);
        assert_ok!(Voting::add_alternatives(RuntimeOrigin::signed(1), 1, 0 , prop_hash));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
			who: bounded_vec![1],
			subjects : bounded_vec![Subject{text :sub_hash}],
			alternatives : bounded_vec![bounded_vec![Alternative{text : prop_hash}]],
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it


	});
}

#[test]
fn vote() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // User 1 create a proposal
		System::assert_last_event(Event::NewProposal { text: hash_text, author: 1, id: 1 }.into()); // We verify the event


		let sub = "sub1";
		let sub_hash = BlakeTwo256::hash_of(&sub);
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash));

        let prop = "prop1";
        let prop_hash = BlakeTwo256::hash_of(&prop);
        assert_ok!(Voting::add_alternatives(RuntimeOrigin::signed(1), 1, 0 , prop_hash));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
            percentage_quorum : 50,
			who: bounded_vec![1],
			subjects : bounded_vec![Subject{text :sub_hash}],
			alternatives : bounded_vec![bounded_vec![Alternative{text : prop_hash}]],
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it

        //let vote : BoundedVec<BoundedVec<u32>> = bounded_vec![bounded_vec![10]];
        assert_ok!(Voting::vote(RuntimeOrigin::signed(1), 1, bounded_vec![bounded_vec![10]]));
        const c :u32 = 100;
        let a : BoundedVec<BoundedVec<u32,c>,c> = bounded_vec![bounded_vec![10]] ; 
        assert_eq!(a, Voting::vote_byid_byquorum((1,1)).unwrap());

	});
}

#[test]
fn finished_vote() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root add two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text));


		assert_ok!(Voting::stop_proposal(RuntimeOrigin::root(), 1)); // Root decided to stop the proposal.

	});
}

