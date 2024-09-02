# Residence DAO

A DAO for managing a housing cooperative or residential building. This project is built using Rust and the Internet Computer platform, enabling efficient decision-making and proposal handling within a community of apartment owners.

## Links

- [Requirements](#requirements)
- [Running the Project Locally](#running-the-project-locally)
- [Commands and Examples](#commands-and-examples)
- [Overview](#overview)
  - [Class Diagram](#class-diagram)
  - [Workflow](#workflow)

## Requirements

To run this project locally, ensure you have the following tools installed:

1. **DFX CLI (0.19.0 or higher)**
   - The DFX CLI is required to create, deploy, and manage canisters on the Internet Computer.
   - [Installing DFX](https://internetcomputer.org/docs/current/developer-docs/getting-started/install/)
   
2. **Rust (rustc 1.80.1 or higher)**
   - Install Rust and Cargo using the official Rust toolchain installer:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
   
3. **WebAssembly Target (wasm32-unknown-unknown)**
   - Add the WebAssembly target to your Rust toolchain to compile Rust code to WebAssembly:
     ```bash
     rustup target add wasm32-unknown-unknown
     ```
   
4. **Candid Extractor**
   - Install the Candid extractor to generate Candid definitions from WebAssembly modules:
     ```bash
     cargo install candid-extractor
     ```

## Running the Project Locally

Follow these steps to start the project on your local environment:

1. **Start the Internet Computer Replica:**
```bash
   dfx start
```
2. **Generate&Deploy the Canister:**
```
  npm run gen-deploy
```

## Commands and Examples

This section provides the essential commands required to operate the DAO for residence management, from the initial deployment to finalizing council members.

### 1. Initial Deployment

When deploying the DAO contract, you need to set up the residence with the following parameters: residence name, the total number of apartments, the builder's identifier (Principal), and any maintenance expenses.

*Parameters:*
- **Residence Name:** The name of the residence.
- **Apartments Count:** The total number of apartments in the residence.
- **Builder Identifier:** The Principal ID of the builder.
- **Expenses:** A list of maintenance expenses.

### 2. Adding Apartments

The builder must add each apartment by calling the `add_apartment` function. This should be done for each apartment, corresponding to the number of apartments defined during the deployment.

**Command:**
`dfx canister call core add_apartment`

*Parameters:*
- **Apartment Number:** A unique number for each apartment.
- **Apartment Name:** The name of the apartment.
- **Owner Identifier:** The Principal ID of the apartment's owner.

### 3. Apply for Council Roles

Apartment owners can apply for council roles using the `apply_for_council` function. Each Principal can apply for only one role to maintain decentralization and security.

**Command:**
`dfx canister call core apply_for_council`

*Parameters:*
- **Apartment Number:** The number of the apartment applying for the role.
- **Council Role:** The role being applied for (Chairman, Treasurer, or Controller).

### 4. Create Council Proposal

After receiving applications, trigger the `make_council_proposal` function to reset any previous votes and prepare the voting process.

**Command:**
`dfx canister call core make_council_proposal`

### 5. Voting on Council Roles

Each apartment owner must vote for every council role. Use the `vote_for_council` function, and refer to `get_council_applications` to review the available applications.

**Command:**
`dfx canister call core vote_for_council`

*Parameters:*
- **Voter Apartment Number:** The number of the apartment casting the vote.
- **Target Apartment Number:** The number of the apartment being voted for.
- **Council Role:** The council role for which the vote is being cast.

### 6. Finalizing the Council

Once all apartments have voted, call the `finalize_council` function to finalize the council members based on the voting results.

**Command:**
`dfx canister call core finalize_council`

### 7. Retrieve Council Members

After finalization, retrieve the list of new council members using the `get_council_members` function.

**Command:**
`dfx canister call core get_council_members`

## Overview

### Class Diagram
The following diagram provides an overview of the main classes and their relationships within the project:

![Class Diagram](img/ClassDiagram.jpg)

### Workflow
The following outline provides a step-by-step process of the main activities and interactions within the DAO for residence management:

1. **Initialize Residence**
   - The residence is initialized with its name, total number of apartments, and any maintenance expenses.
   - Declare the builder and set the maintenance expenses.

2. **Builder Adds Apartments**
   - The builder adds apartments to the residence, associating each with its respective owner.

3. **Owners Apply for Council Roles**
   - Apartment owners apply for council roles such as Chairman, Treasurer, or Controller.
   - Store applications submitted by the owners.

4. **Create Council Proposal**
   - A proposal is created based on the applications received.
   - Reset any previous votes and collect new votes from the owners.

5. **Owners Vote on Council Roles**
   - Owners cast their votes on the council roles based on the proposal.

6. **Validate Votes**
   - The system checks if all apartments have voted for every role.

7. **Check Voting Completion**
   - Determine whether all apartments have voted.
   - If not all apartments have voted, return to the voting process.

8. **Finalize Council Members**
   - Once all votes are validated, finalize the new council members based on the voting results.
   - Determine the winning apartments for each role.

9. **Set New Council Members**
    - Set the new council members according to the voting results.

10. **End**
    - The DAO process for the current cycle concludes, and the council members are set for the new term.