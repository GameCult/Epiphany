#[test]
fn mutation_routes_are_explicit_compatibility_delegates() {
    let source = include_str!("epiphany_mutation_routes.rs");
    for delegate in [
        "apply_thread_epiphany_update(",
        "interrupt_thread_epiphany_job(",
    ] {
        assert!(source.contains(delegate), "missing compatibility delegate {delegate}");
    }
    for forbidden in [
        "apply_epiphany_state_update_to_state(",
        "acceptance_receipts.push(",
        "job_bindings.push(",
        "runtime_links.push(",
        ".steer_input(",
        ".submit(Op::Compact)",
        "select_epiphany_coordinator_automation(",
    ] {
        assert!(
            !source.contains(forbidden),
            "compatibility route regained local authority through {forbidden}"
        );
    }
}

#[test]
fn only_named_client_routes_remain_registered() {
    let protocol = include_str!("../../../app-server-protocol/src/protocol/common.rs");
    let registrations = protocol
        .split("ThreadTurnsList =>")
        .next()
        .expect("client request registration section");
    for retained in [
        "thread/epiphany/view",
        "thread/epiphany/roleResult",
        "thread/epiphany/freshness",
        "thread/epiphany/context",
        "thread/epiphany/graphQuery",
        "thread/epiphany/reorientResult",
        "thread/epiphany/jobInterrupt",
        "thread/epiphany/update",
    ] {
        assert!(registrations.contains(retained), "missing retained client route {retained}");
    }
    for removed in [
        "thread/epiphany/roleLaunch",
        "thread/epiphany/roleAccept",
        "thread/epiphany/reorientLaunch",
        "thread/epiphany/reorientAccept",
        "thread/epiphany/index",
        "thread/epiphany/distill",
        "thread/epiphany/propose",
        "thread/epiphany/promote",
        "thread/epiphany/jobLaunch",
        "thread/epiphany/retrieve",
    ] {
        assert!(!registrations.contains(removed), "unconsumed route remains registered: {removed}");
    }
}

#[test]
fn read_routes_have_no_mutation_or_scheduler_authority() {
    let source = include_str!("epiphany_read_routes.rs");
    for forbidden in [
        "apply_thread_epiphany_",
        "launch_thread_epiphany_",
        "interrupt_thread_epiphany_",
        "index_epiphany_retrieval_for_paths(",
        ".steer_input(",
        ".submit(Op::Compact)",
        "select_epiphany_coordinator_automation(",
    ] {
        assert!(
            !source.contains(forbidden),
            "read route regained actuator authority through {forbidden}"
        );
    }
}

#[test]
fn compatibility_surface_has_no_implicit_event_loop_trigger() {
    let processor = include_str!("../codex_message_processor.rs");
    let event_loop = include_str!("../bespoke_event_handling.rs");
    let thread_state = include_str!("../thread_state.rs");

    assert!(!processor.contains("mod epiphany_automation"));
    assert!(!event_loop.contains("maybe_run_epiphany"));
    assert!(!thread_state.contains("epiphany_checkpoint_intervention"));
}
