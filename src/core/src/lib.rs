// Import necessary external crates and modules
#[macro_use]
extern crate serde;
use candid::{Decode, Encode, Principal, CandidType};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use ic_cdk::{api};
use std::{borrow::Cow, cell::RefCell};

// Define type aliases for memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;

// Macro to implement Storable and BoundedStorable traits for custom types
macro_rules! impl_storable_and_bounded {
    ($t:ty, $max_size:expr) => {
        impl Storable for $t {
            fn to_bytes(&self) -> Cow<[u8]> {
                Cow::Owned(Encode!(self).unwrap())
            }

            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                Decode!(bytes.as_ref(), Self).unwrap()
            }
        }

        impl BoundedStorable for $t {
            const MAX_SIZE: u32 = $max_size;
            const IS_FIXED_SIZE: bool = false;
        }
    };
}

// Struct Definitions and Default Trait Implementations
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Builder {
    id: Principal,
    name: String,
    contact_info: String,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            id: Principal::anonymous(), // Use Principal::anonymous as a default value
            name: String::from(""),
            contact_info: String::from(""),
        }
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Residence {
    name: String,
    apartments_count: u32,
    maintenance_expenses: Vec<MaintenanceExpense>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct MaintenanceExpense {
    name: String,
    amount: f64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Apartment {
    name: String,
    number: u32,
    owner: Principal,
}

impl Default for Apartment {
    fn default() -> Self {
        Apartment {
            name: String::from(""),
            number: 0,
            owner: Principal::anonymous(),
        }
    }
}

impl_storable_and_bounded!(Apartment, 128);

#[derive(candid::CandidType, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum CouncilRole {
    Chairman,
    Treasurer,
    Controller,
}

impl Default for CouncilRole {
    fn default() -> Self {
        CouncilRole::Chairman
    }
}

impl_storable_and_bounded!(CouncilRole, 128);

#[derive(Clone, Serialize, Deserialize, Debug, CandidType, Default)]
struct CouncilVoteEntry {
    apartment_number: u32,
    votes: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, CandidType)]
struct CouncilVotes {
    chairman_votes: Vec<CouncilVoteEntry>,
    treasurer_votes: Vec<CouncilVoteEntry>,
    controller_votes: Vec<CouncilVoteEntry>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, CandidType)]
struct PrincipalWrapper(Principal);

impl_storable_and_bounded!(PrincipalWrapper, 128);

#[derive(Clone, Copy, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
struct BoolWrapper(bool);

impl Storable for BoolWrapper {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(vec![self.0 as u8])
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        BoolWrapper(bytes[0] != 0)
    }
}

impl BoundedStorable for BoolWrapper {
    const MAX_SIZE: u32 = 1;
    const IS_FIXED_SIZE: bool = true;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct CouncilApplication {
    apartment_number: u32,
    role: CouncilRole,
}

impl_storable_and_bounded!(CouncilApplication, 128);

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InsufficientFunds { msg: String },
}

// Define Global State for Managing DAO data
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static RESIDENCE: RefCell<Residence> = RefCell::new(Residence::default());
    static BUILDER: RefCell<Builder> = RefCell::new(Builder::default());

    static APARTMENT_STORAGE: RefCell<StableBTreeMap<u32, Apartment, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    static COUNCIL_APPLICATIONS: RefCell<StableBTreeMap<PrincipalWrapper, CouncilApplication, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    static COUNCIL_MEMBERS: RefCell<StableBTreeMap<CouncilRole, PrincipalWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
        )
    );

    static COUNCIL_VOTES: RefCell<CouncilVotes> = RefCell::new(CouncilVotes {
        chairman_votes: Vec::new(),
        treasurer_votes: Vec::new(),
        controller_votes: Vec::new(),
    });

    static VOTED_APARTMENTS: RefCell<StableBTreeMap<(u32, CouncilRole), BoolWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
        )
    );
}

// Initialization function for the dApp, setting up the initial state
#[ic_cdk::init]
fn init(residence_name: String, apartments_count: u32, builder: Builder, maintenance_expenses: Vec<MaintenanceExpense>) {
    // Validate residence name
    if residence_name.is_empty() {
        panic!("Residence name cannot be empty.");
    }

    // Validate apartments count
    if apartments_count == 0 {
        panic!("Apartments count must be greater than zero.");
    }

    // Optionally, validate maintenance expenses (assuming it can't be empty if provided)
    if maintenance_expenses.is_empty() {
        panic!("Maintenance expenses cannot be empty.");
    }

    let residence_name_clone = residence_name.clone();

    RESIDENCE.with(|residence| {
        let mut residence = residence.borrow_mut();
        residence.name = residence_name;
        residence.apartments_count = apartments_count;
        residence.maintenance_expenses = maintenance_expenses;
    });

    BUILDER.with(|builder_cell| {
        let mut builder_cell = builder_cell.borrow_mut();
        *builder_cell = builder;
    });

    ic_cdk::println!(
        "DAO initialized with residence name: {} and apartments count: {}",
        residence_name_clone, apartments_count
    );
}

// Query function to get the current residence state
#[ic_cdk::query]
fn get_residence() -> Residence {
    RESIDENCE.with(|residence| {
        residence.borrow().clone()
    })
}

// Update function to add a new apartment to the storage
#[ic_cdk::update]
fn add_apartment(apartment_number: u32, apartment_name: String, owner: Principal) -> Result<(), String> {
    // Check if the caller is the Builder
    let caller = api::caller();
    let is_builder = BUILDER.with(|builder| builder.borrow().id == caller);

    if !is_builder {
        return Err("Only the builder can add apartments.".to_string());
    }

    // Validate apartment number
    if apartment_number == 0 {
        return Err("Apartment number cannot be zero.".to_string());
    }

    // Validate apartment name
    if apartment_name.is_empty() {
        return Err("Apartment name cannot be empty.".to_string());
    }

    let max_apartments = RESIDENCE.with(|residence| residence.borrow().apartments_count);
    let current_apartments_count = APARTMENT_STORAGE.with(|storage| storage.borrow().len() as u32);

    if current_apartments_count >= max_apartments {
        return Err(format!("Cannot add more than {} apartments.", max_apartments));
    }

    let apartment_exists = APARTMENT_STORAGE.with(|storage| storage.borrow().contains_key(&apartment_number));

    if apartment_exists {
        return Err(format!("Apartment number {} is already added.", apartment_number));
    }

    let apartment = Apartment {
        name: apartment_name,
        number: apartment_number,
        owner,
    };

    APARTMENT_STORAGE.with(|storage| {
        storage.borrow_mut().insert(apartment_number, apartment);
    });

    Ok(())
}

// Query function to get the list of all apartments
#[ic_cdk::query]
fn get_apartments() -> Vec<Apartment> {
    APARTMENT_STORAGE.with(|storage| {
        storage.borrow().iter().map(|(_, apartment)| apartment.clone()).collect()
    })
}

// Update function to apply for a council role by an apartment owner
#[ic_cdk::update]
fn apply_for_council(apartment_number: u32, role: CouncilRole) -> Result<(), String> {
    let caller = api::caller(); // Get the caller's principal

    let applicant_owner_id = APARTMENT_STORAGE.with(|storage| {
        storage.borrow().get(&apartment_number).map(|apt| PrincipalWrapper(apt.owner))
    });

    match applicant_owner_id {
        Some(owner_id) => {
            // Check if the caller is the owner of the apartment
            if owner_id.0 != caller {
                return Err("You can only apply for a council role for an apartment you own.".to_string());
            }

            // Check if the owner has already applied for the specific role
            let already_applied_for_role = COUNCIL_APPLICATIONS.with(|applications| {
                applications.borrow().iter().any(|(owner, app)| {
                    owner == owner_id && app.role == role
                })
            });

            if already_applied_for_role {
                return Err(format!("Owner of apartment {} has already applied for the role of {:?}.", apartment_number, role));
            }

            // Add the application to the storage
            COUNCIL_APPLICATIONS.with(|applications| {
                applications.borrow_mut().insert(owner_id, CouncilApplication { apartment_number, role });
            });

            Ok(())
        }
        None => Err(format!("Apartment {} does not exist.", apartment_number)),
    }
}

// Query function to get the list of council applications
#[ic_cdk::query]
fn get_council_applications() -> Vec<(PrincipalWrapper, u32, CouncilRole)> {
    COUNCIL_APPLICATIONS.with(|applications| {
        applications.borrow().iter().map(|(owner_id, app)| (owner_id.clone(), app.apartment_number, app.role.clone())).collect()
    })
}

// Query function to get the current state of council votes
#[ic_cdk::query]
fn get_council_votes() -> CouncilVotes {
    COUNCIL_VOTES.with(|votes| votes.borrow().clone())
}

// Query function to get the current council members
#[ic_cdk::query]
fn get_council_members() -> Vec<(CouncilRole, PrincipalWrapper)> {
    COUNCIL_MEMBERS.with(|members| {
        members.borrow().iter().map(|(role, owner)| (role.clone(), owner.clone())).collect()
    })
}

// Update function to propose a new council by resetting the votes and setting up new candidates
#[ic_cdk::update]
fn make_council_proposal() -> Result<(), String> {
    // Check if VOTED_APARTMENTS is empty
    let is_voted_apartments_empty = VOTED_APARTMENTS.with(|voted_apartments| {
        voted_apartments.borrow().is_empty()
    });

    if !is_voted_apartments_empty {
        return Err("Council proposal cannot be made because there are existing votes.".to_string());
    }
    
    // Reset votes for new proposal
    COUNCIL_VOTES.with(|votes| {
        let mut votes = votes.borrow_mut();
        votes.chairman_votes.clear();
        votes.treasurer_votes.clear();
        votes.controller_votes.clear();
    });

    // Populate votes with applications
    COUNCIL_APPLICATIONS.with(|applications| {
        let applications = applications.borrow();
        applications.iter().for_each(|(owner_id, application)| {
            let apartment_number = APARTMENT_STORAGE.with(|storage| {
                storage.borrow().iter().find(|(_, apt)| PrincipalWrapper(apt.owner) == owner_id).map(|(num, _)| num)
            });

            if let Some(apartment_number) = apartment_number {
                COUNCIL_VOTES.with(|votes| {
                    let mut votes = votes.borrow_mut();
                    match application.role {
                        CouncilRole::Chairman => votes.chairman_votes.push(CouncilVoteEntry { apartment_number, votes: 0 }),
                        CouncilRole::Treasurer => votes.treasurer_votes.push(CouncilVoteEntry { apartment_number, votes: 0 }),
                        CouncilRole::Controller => votes.controller_votes.push(CouncilVoteEntry { apartment_number, votes: 0 }),
                    }
                });
            }
        });
    });

    Ok(())
}

// Update function to cast a vote for a council role
#[ic_cdk::update]
fn vote_for_council(voter_apartment_number: u32, target_apartment_number: u32, role: CouncilRole) -> Result<(), String> {
    // Validate that the caller is the owner of the voting apartment
    let caller = api::caller();
    let is_owner = APARTMENT_STORAGE.with(|storage| {
        storage.borrow().get(&voter_apartment_number).map_or(false, |apt| apt.owner == caller)
    });

    if !is_owner {
        return Err("You can only vote from an apartment you own.".to_string());
    }

    // Check if the apartment has already voted for this role
    let has_voted = VOTED_APARTMENTS.with(|voted_apartments| {
        voted_apartments.borrow().contains_key(&(voter_apartment_number, role.clone()))
    });

    if has_voted {
        return Err("This apartment has already voted for this role.".to_string());
    }

    // Proceed with voting
    COUNCIL_VOTES.with(|votes| {
        let mut votes = votes.borrow_mut();
        let vote_list = match role {
            CouncilRole::Chairman => &mut votes.chairman_votes,
            CouncilRole::Treasurer => &mut votes.treasurer_votes,
            CouncilRole::Controller => &mut votes.controller_votes,
        };

        for vote in vote_list.iter_mut() {
            if vote.apartment_number == target_apartment_number {
                vote.votes += 1;

                // Mark this apartment as having voted for this role
                VOTED_APARTMENTS.with(|voted_apartments| {
                    voted_apartments.borrow_mut().insert((voter_apartment_number, role), BoolWrapper(true));
                });

                return Ok(());
            }
        }

        Err("No such apartment in the council applications.".to_string())
    })
}

// Update function to finalize the council after all votes are cast
#[ic_cdk::update]
fn finalize_council() -> Result<(), String> {
    // Validate if all apartments have voted for every role
    let all_apartments_voted = APARTMENT_STORAGE.with(|storage| {
        let mut all_voted = true;

        // Iterate over each apartment and check if it has voted for each role
        for (apartment_number, _) in storage.borrow().iter() {
            let voted_for_chairman = VOTED_APARTMENTS.with(|voted_apartments| {
                voted_apartments.borrow().contains_key(&(apartment_number, CouncilRole::Chairman))
            });

            let voted_for_treasurer = VOTED_APARTMENTS.with(|voted_apartments| {
                voted_apartments.borrow().contains_key(&(apartment_number, CouncilRole::Treasurer))
            });

            let voted_for_controller = VOTED_APARTMENTS.with(|voted_apartments| {
                voted_apartments.borrow().contains_key(&(apartment_number, CouncilRole::Controller))
            });

            if !voted_for_chairman || !voted_for_treasurer || !voted_for_controller {
                all_voted = false;
                break;
            }
        }

        all_voted
    });

    if !all_apartments_voted {
        return Err("Not all apartments have voted for every role.".to_string());
    }

    let (chairman_apartment_number, treasurer_apartment_number, controller_apartment_number) =
        COUNCIL_VOTES.with(|votes| {
            let votes = votes.borrow();

            // Ensure all roles have valid votes
            if votes.chairman_votes.is_empty() || votes.treasurer_votes.is_empty() || votes.controller_votes.is_empty() {
                return Err("Not all roles have been voted for. Please ensure all roles have votes before finalizing.".to_string());
            }

            // Determine winners for each role
            let chairman_apartment_number = determine_council_role_winner(&votes.chairman_votes)?;
            let treasurer_apartment_number = determine_council_role_winner(&votes.treasurer_votes)?;
            let controller_apartment_number = determine_council_role_winner(&votes.controller_votes)?;

            Ok((
                chairman_apartment_number,
                treasurer_apartment_number,
                controller_apartment_number,
            ))
        })?;

    // Reset and set new council members
    COUNCIL_MEMBERS.with(|members| {
        let mut members = members.borrow_mut();

        // Manually remove each key from the map
        members.remove(&CouncilRole::Chairman);
        members.remove(&CouncilRole::Treasurer);
        members.remove(&CouncilRole::Controller);

        // Set new council members
        if let Some(chairman_owner) = APARTMENT_STORAGE.with(|storage| {
            storage.borrow().get(&chairman_apartment_number).map(|apt| PrincipalWrapper(apt.owner))
        }) {
            members.insert(CouncilRole::Chairman, chairman_owner);
        } else {
            return Err("Chairman apartment not found.".to_string());
        }

        if let Some(treasurer_owner) = APARTMENT_STORAGE.with(|storage| {
            storage.borrow().get(&treasurer_apartment_number).map(|apt| PrincipalWrapper(apt.owner))
        }) {
            members.insert(CouncilRole::Treasurer, treasurer_owner);
        } else {
            return Err("Treasurer apartment not found.".to_string());
        }

        if let Some(controller_owner) = APARTMENT_STORAGE.with(|storage| {
            storage.borrow().get(&controller_apartment_number).map(|apt| PrincipalWrapper(apt.owner))
        }) {
            members.insert(CouncilRole::Controller, controller_owner);
        } else {
            return Err("Controller apartment not found.".to_string());
        }

        Ok(())
    })?;

    // Clear votes after successful finalization
    COUNCIL_VOTES.with(|votes| {
        let mut votes = votes.borrow_mut();
        votes.chairman_votes.clear();
        votes.treasurer_votes.clear();
        votes.controller_votes.clear();
    });

    // Clear voted apartments map
    VOTED_APARTMENTS.with(|voted_apartments| {
        let mut voted_apartments = voted_apartments.borrow_mut();
        let keys: Vec<_> = voted_apartments.iter().map(|(key, _)| key.clone()).collect();
        for key in keys {
            voted_apartments.remove(&key);
        }
    });

    Ok(())
}

// Function to determine the winner of a council role based on votes
fn determine_council_role_winner(votes: &[CouncilVoteEntry]) -> Result<u32, String> {
    if votes.is_empty() {
        return Err("No candidates for this role.".to_string());
    }

    let max_votes = votes.iter().max_by_key(|v| v.votes).unwrap().votes;
    let candidates: Vec<_> = votes.iter().filter(|v| v.votes == max_votes).collect();

    if candidates.len() > 1 {
        // If there is a tie, return an error to resolve the tie manually
        return Err("There is a tie between candidates. Please resolve manually.".to_string());
    }

    Ok(candidates[0].apartment_number)
}

// Update function to return the caller's principal (identity)
#[ic_cdk::update]
fn whoami() -> Result<Principal, Error> {
    let caller = api::caller();

    return  Ok(caller);
}


// Export the candid interface for the dApp
ic_cdk::export_candid!();
