#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol};

#[contract]
pub struct StudyCircleFund;

#[contractimpl]
impl StudyCircleFund {
    /// Initialize the contract with an admin who can manage global parameters.
    /// Must be called once before any circle is created.
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        if env
            .storage()
            .instance()
            .get::<_, bool>(&"initialized")
            .is_some()
        {
            panic!("Already initialized");
        }
        env.storage().instance().set(&"admin", &admin);
        env.storage().instance().set(&"initialized", &true);
        env.storage().instance().set(&"next_circle_id", &1u64);
        env.storage().instance().set(&"next_request_id", &1u64);
    }

    /// Create a new study circle. The founder is auto-enrolled as the first
    /// member and a unique `circle_id` is returned.
    pub fn create_circle(env: Env, founder: Address, name: Symbol) -> u64 {
        founder.require_auth();

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&"next_circle_id")
            .unwrap_or(1u64);
        let circle_id = next_id;

        let mut members: Map<Address, bool> = Map::new(&env);
        members.set(founder.clone(), true);

        env.storage()
            .instance()
            .set(&("members", circle_id), &members);
        env.storage()
            .instance()
            .set(&("founder", circle_id), &founder);
        env.storage().instance().set(&("name", circle_id), &name);
        env.storage()
            .instance()
            .set(&("balance", circle_id), &0i128);
        env.storage().instance().set(&("active", circle_id), &true);
        env.storage()
            .instance()
            .set(&"next_circle_id", &(next_id + 1u64));

        env.events().publish(
            (Symbol::new(&env, "circle_created"),),
            (circle_id, founder, name),
        );

        circle_id
    }

    /// Record a contribution from a member to a study circle's pooled
    /// balance. The contributor is auto-enrolled as a member if they were
    /// not one already. Returns the new circle balance.
    pub fn contribute(env: Env, member: Address, circle_id: u64, amount: i128) -> i128 {
        member.require_auth();

        if amount <= 0 {
            panic!("Contribution must be positive");
        }

        let active: bool = env
            .storage()
            .instance()
            .get(&("active", circle_id))
            .expect("Circle does not exist");
        if !active {
            panic!("Circle is not active");
        }

        // Auto-enroll the contributor as a member.
        let mut members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&("members", circle_id))
            .unwrap_or(Map::new(&env));
        members.set(member.clone(), true);
        env.storage()
            .instance()
            .set(&("members", circle_id), &members);

        // Credit the circle's pooled balance.
        let balance: i128 = env
            .storage()
            .instance()
            .get(&("balance", circle_id))
            .unwrap_or(0i128);
        let new_balance = balance + amount;
        env.storage()
            .instance()
            .set(&("balance", circle_id), &new_balance);

        // Track per-member contribution totals so we can reward heavy
        // contributors later (e.g. weighted voting).
        let key = ("contribution", circle_id, member.clone());
        let prev: i128 = env.storage().instance().get(&key).unwrap_or(0i128);
        env.storage().instance().set(&key, &(prev + amount));

        env.events().publish(
            (Symbol::new(&env, "contributed"),),
            (circle_id, member, amount, new_balance),
        );

        new_balance
    }

    /// Submit a spending request for shared resources such as books, lab
    /// fees, software licenses, or tutoring. The request is queued and
    /// needs member approvals before any funds are debited. Returns the
    /// newly assigned `request_id`.
    pub fn request_disbursement(
        env: Env,
        requester: Address,
        circle_id: u64,
        purpose: Symbol,
        amount: i128,
    ) -> u64 {
        requester.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let active: bool = env
            .storage()
            .instance()
            .get(&("active", circle_id))
            .expect("Circle does not exist");
        if !active {
            panic!("Circle is not active");
        }

        let balance: i128 = env
            .storage()
            .instance()
            .get(&("balance", circle_id))
            .unwrap_or(0i128);
        if amount > balance {
            panic!("Requested amount exceeds circle balance");
        }

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&"next_request_id")
            .unwrap_or(1u64);
        let request_id = next_id;

        env.storage()
            .instance()
            .set(&("req_circle", request_id), &circle_id);
        env.storage()
            .instance()
            .set(&("req_requester", request_id), &requester);
        env.storage()
            .instance()
            .set(&("req_purpose", request_id), &purpose);
        env.storage()
            .instance()
            .set(&("req_amount", request_id), &amount);
        env.storage()
            .instance()
            .set(&("req_approvals", request_id), &0u32);
        // Default governance: 2 approvals required to disburse funds.
        env.storage()
            .instance()
            .set(&("req_required", request_id), &2u32);
        env.storage()
            .instance()
            .set(&("req_executed", request_id), &false);
        env.storage()
            .instance()
            .set(&"next_request_id", &(next_id + 1u64));

        env.events().publish(
            (Symbol::new(&env, "disbursement_requested"),),
            (request_id, circle_id, requester, purpose, amount),
        );

        request_id
    }

    /// Approve a pending disbursement request. Once the approval threshold
    /// (default 2) is reached, the request auto-executes and the circle's
    /// pooled balance is debited. The requester is not allowed to approve
    /// their own request, and a member can only approve once per request.
    /// Returns `true` if this approval caused the request to execute.
    pub fn approve_disbursement(env: Env, approver: Address, request_id: u64) -> bool {
        approver.require_auth();

        let executed: bool = env
            .storage()
            .instance()
            .get(&("req_executed", request_id))
            .expect("Request does not exist");
        if executed {
            panic!("Request already executed");
        }

        let circle_id: u64 = env
            .storage()
            .instance()
            .get(&("req_circle", request_id))
            .expect("Request does not exist");
        let requester: Address = env
            .storage()
            .instance()
            .get(&("req_requester", request_id))
            .expect("Request does not exist");
        let amount: i128 = env
            .storage()
            .instance()
            .get(&("req_amount", request_id))
            .expect("Request does not exist");

        if approver == requester {
            panic!("Requester cannot approve their own request");
        }

        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&("members", circle_id))
            .unwrap_or(Map::new(&env));
        if !members.get(approver.clone()).unwrap_or(false) {
            panic!("Approver is not a member of the circle");
        }

        let approval_key = ("approval", request_id, approver.clone());
        if env
            .storage()
            .instance()
            .get::<_, bool>(&approval_key)
            .unwrap_or(false)
        {
            panic!("Already approved by this member");
        }
        env.storage().instance().set(&approval_key, &true);

        let approvals: u32 = env
            .storage()
            .instance()
            .get(&("req_approvals", request_id))
            .unwrap_or(0u32);
        let new_approvals = approvals + 1u32;
        env.storage()
            .instance()
            .set(&("req_approvals", request_id), &new_approvals);

        let required: u32 = env
            .storage()
            .instance()
            .get(&("req_required", request_id))
            .unwrap_or(1u32);

        let mut executed_now = false;
        if new_approvals >= required {
            let balance: i128 = env
                .storage()
                .instance()
                .get(&("balance", circle_id))
                .unwrap_or(0i128);
            if amount > balance {
                panic!("Circle balance insufficient at execution time");
            }
            env.storage()
                .instance()
                .set(&("balance", circle_id), &(balance - amount));
            env.storage()
                .instance()
                .set(&("req_executed", request_id), &true);
            executed_now = true;
        }

        env.events().publish(
            (Symbol::new(&env, "disbursement_approved"),),
            (request_id, approver, new_approvals, executed_now),
        );

        executed_now
    }

    /// Read the current pooled balance of a study circle (0 if the circle
    /// does not exist).
    pub fn get_balance(env: Env, circle_id: u64) -> i128 {
        env.storage()
            .instance()
            .get(&("balance", circle_id))
            .unwrap_or(0i128)
    }

    /// Read the total amount a member has contributed to a specific study
    /// circle.
    pub fn get_member_contribution(env: Env, circle_id: u64, member: Address) -> i128 {
        let key = ("contribution", circle_id, member);
        env.storage().instance().get(&key).unwrap_or(0i128)
    }

    /// Check whether a given address is enrolled as a member of a study
    /// circle.
    pub fn is_member(env: Env, circle_id: u64, member: Address) -> bool {
        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&("members", circle_id))
            .unwrap_or(Map::new(&env));
        members.get(member).unwrap_or(false)
    }

    /// Get the approval status of a disbursement request as
    /// `(approvals_so_far, required_approvals, executed)`.
    pub fn get_request_status(env: Env, request_id: u64) -> (u32, u32, bool) {
        let approvals: u32 = env
            .storage()
            .instance()
            .get(&("req_approvals", request_id))
            .unwrap_or(0u32);
        let required: u32 = env
            .storage()
            .instance()
            .get(&("req_required", request_id))
            .unwrap_or(0u32);
        let executed: bool = env
            .storage()
            .instance()
            .get(&("req_executed", request_id))
            .unwrap_or(false);
        (approvals, required, executed)
    }
}
