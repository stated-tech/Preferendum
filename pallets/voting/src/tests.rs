use crate::{mock::*, Alternative, Event, Proposal, ProposalStatus, Subject};
use frame_support::{assert_ok, traits::ConstU32, BoundedVec};
use sp_core::bounded_vec;
use sp_runtime::traits::{BlakeTwo256, Hash};
type MaxSubjects = ConstU32<100>;

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
			percentage_quorum: 50,
			who: bounded_vec![1],
			subjects: Default::default(),
			alternatives: Default::default(),
		};

		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it
	});
}

#[test]
fn add_someone() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root added a user
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root added a user

		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist

		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(2), 1)); // We add a second user in the quorum

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1, 2], // There is 2 people
			subjects: Default::default(),
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
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash)); // Assert one subject in added

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1],
			subjects: bounded_vec![Subject { text: sub_hash }], // Subject added
			alternatives: bounded_vec![Default::default()],
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

		let alt = "prop1";
		let alt_hash = BlakeTwo256::hash_of(&alt);
		assert_ok!(Voting::add_alternatives(RuntimeOrigin::signed(1), 1, 0, alt_hash)); // Add an alternative

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1],
			subjects: bounded_vec![Subject { text: sub_hash }],
			alternatives: bounded_vec![bounded_vec![Alternative { text: alt_hash }]],
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it
	});
}

#[test]
fn change_status_to_vote() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root create a voter "1"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root create a voter "2"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 3)); // Root create a voter "3"

		assert_eq!(Voting::registered_voters(1), Some(())); // We make sure he is inside

		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(2), 1)); // Add voters 2 and 3
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(3), 1));

		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(1), 1)); // Vote of only one person

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1, 2, 3],
			subjects: Default::default(),
			alternatives: Default::default(),
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // The status has not changed because the quorum is not enough

		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(2), 1)); // A second one vote
		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Vote, // The status has changed !
			percentage_quorum: 50,
			who: bounded_vec![1, 2, 3],
			subjects: Default::default(),
			alternatives: Default::default(),
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it

		let sub = "sub1";
		let sub_hash = BlakeTwo256::hash_of(&sub);
		let addsub = Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash);
		assert!(addsub.is_err()); // Make sure not possible to vote again because the status has changed
	});
}

#[test]
fn change_status_to_stop_vote() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root create a voter "1"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root create a voter "2"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 3)); // Root create a voter "3"

		assert_eq!(Voting::registered_voters(1), Some(())); // We make sure he is inside

		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(2), 1)); // Add two more people
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(3), 1));

		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(1), 1)); // Two people vote
		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(2), 1));

		assert_ok!(Voting::quorum_propose_stop_vote(RuntimeOrigin::signed(2), 1)); // Two propose to stop the vote
		assert_ok!(Voting::quorum_propose_stop_vote(RuntimeOrigin::signed(1), 1));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Finished, // The status is nos Finished
			percentage_quorum: 50,
			who: bounded_vec![1, 2, 3],
			subjects: Default::default(),
			alternatives: Default::default(),
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it

		let sub = "sub1";
		let sub_hash = BlakeTwo256::hash_of(&sub);
		let addsub = Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash);
		assert!(addsub.is_err()); // Make sure not possible to vote again
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

		let sub = "sub1"; // Add subject and alternative
		let sub_hash = BlakeTwo256::hash_of(&sub);
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash));

		let prop = "prop1";
		let prop_hash = BlakeTwo256::hash_of(&prop);
		assert_ok!(Voting::add_alternatives(RuntimeOrigin::signed(1), 1, 0, prop_hash));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1],
			subjects: bounded_vec![Subject { text: sub_hash }],
			alternatives: bounded_vec![bounded_vec![Alternative { text: prop_hash }]], /* Alternative added */
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(2), 1));

		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(1), 1)); // Propose vote
		assert_ok!(Voting::quorum_propose_vote(RuntimeOrigin::signed(2), 1));
		assert_ok!(Voting::vote(RuntimeOrigin::signed(1), 1, bounded_vec![bounded_vec![10]])); // User 1 vote

		let vote: BoundedVec<BoundedVec<u32, MaxSubjects>, MaxSubjects> =
			bounded_vec![bounded_vec![10]]; // Vote made
		assert_eq!(vote, Voting::vote_byid_byquorum((1, 1)).unwrap()); // Make sure it is ok
	});
}

#[test]
fn kick_someone_quorum() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1)); // Root create a voter "1"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 2)); // Root create a voter "2"
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 3)); // Root create a voter "3"

		assert_eq!(Voting::registered_voters(1), Some(())); // We make sure he is inside

		let text = "hello"; // text of the proposal
		let hash_text = BlakeTwo256::hash_of(&text); //text hashed
		assert_ok!(Voting::create_proposal(RuntimeOrigin::signed(1), hash_text)); // we make sure the proposal exist
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(2), 1)); // Add two more people
		assert_ok!(Voting::add_voters(RuntimeOrigin::signed(3), 1));

		let sub = "sub1"; // Subject and alternative is added
		let sub_hash = BlakeTwo256::hash_of(&sub);
		assert_ok!(Voting::add_subject(RuntimeOrigin::signed(1), 1, sub_hash));

		let prop = "prop1";
		let prop_hash = BlakeTwo256::hash_of(&prop);
		assert_ok!(Voting::add_alternatives(RuntimeOrigin::signed(1), 1, 0, prop_hash));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1, 2, 3],
			subjects: bounded_vec![Subject { text: sub_hash }],
			alternatives: bounded_vec![bounded_vec![Alternative { text: prop_hash }]],
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it

		assert_ok!(Voting::quorum_kick_proposal(RuntimeOrigin::signed(1), 1, 0, 0)); // Two persob kick an alternative
		assert_ok!(Voting::quorum_kick_proposal(RuntimeOrigin::signed(2), 1, 0, 0));

		let expected_proposal = Proposal::<Test> {
			text: hash_text,
			id: 1,
			progress: ProposalStatus::Inprogress,
			percentage_quorum: 50,
			who: bounded_vec![1, 2, 3],
			subjects: bounded_vec![Subject { text: sub_hash }],
			alternatives: bounded_vec![bounded_vec![]], // No more alternative
		};
		assert_eq!(Voting::proposals(1).unwrap(), expected_proposal); // Make sure we get it
	});
}
