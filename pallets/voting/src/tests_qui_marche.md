use crate::{mock::*, Event, Proposal, ProposalStatus, Vote , Subject};
use frame_support::{assert_ok};
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

		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist
		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			who: Default::default(),
			amount: Default::default(),
			subjects : Default::default(),
			propositions: Default::default(),
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
			who: Default::default(),
			amount: Default::default(),
			subjects : bounded_vec![Subject{text :sub_hash}],
			propositions : Default::default(),
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it


	});
}



#[test]
fn create_vote_() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // User 1 create a proposal
		System::assert_last_event(Event::NewProposal { text: hash_text, author: 1, id: 1 }.into()); // We verify the event

		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(1), 1, Vote::Aye(20))); // User 1 make a vote
		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(2), 1, Vote::Aye(20))); // User 2 make a vote

		assert_eq!(Voting::aye_vote(1), Some(40)); // we add the two vote

	});
}

#[test]
fn cant_vote_twice() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // User 1 create a proposal
		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(1), 1, Vote::Aye(20))); // User 1 make a vote

		let second_vote = Voting::create_vote(RuntimeOrigin::signed(1), 1, Vote::Aye(20)); // User 2 make a vote
		assert!(second_vote.is_err());
		assert_eq!(Voting::aye_vote(1), Some(20)); // we don't add the two vote, only one is take into
		                                   // account
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

		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(1), 1, Vote::Aye(20)));

		assert_eq!(Voting::aye_vote(1), Some(20)); // Make sure the vote took into account

		assert_ok!(Voting::stop_proposal(RuntimeOrigin::root(), 1)); // Root decided to stop the proposal.
		let second_vote = Voting::create_vote(RuntimeOrigin::signed(2), 1, Vote::Aye(20)); // User 2 make a vote
		assert!(second_vote.is_err()); // we tried to make a vote of the user 2 after the end but it is not
		                       // possible
		assert!(Voting::aye_vote(1)>Voting::nay_vote(1)); // We verify that the proposal get more aye than nay
	});
}

#[test]
fn unlock_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root add two users
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2));

		let text = "hello";
		let hash_text = BlakeTwo256::hash_of(&text);
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text));
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(2), hash_text)); // User 2 create a second proposal

		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(1), 1, Vote::Aye(20))); // User vote all its balance on first proposal
		let second_vote = Voting::create_vote(RuntimeOrigin::signed(1), 2, Vote::Aye(10));
		assert!(second_vote.is_err()); // It should be an error because not enought balance

		assert_ok!(Voting::stop_proposal(RuntimeOrigin::root(), 1)); // Root stop the first proposal, so User one get back 400 (20²)

		assert_ok!(Voting::create_vote(RuntimeOrigin::signed(1), 2, Vote::Aye(10)));
		assert_eq!(Voting::aye_vote(2), Some(10)); // User 1 can vote and it works


	});
	
}


