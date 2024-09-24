
    // note: use edge->is_fw(bw) instead of edge->weight_forward(backward)

    // forward pass

    // for every factor A in seq.arr, do the following:
    // - for every incoming edge e that is_backward, send message along e
    // - if compute_solution, compute solution
    // - let theta be the current reparametrization of A
    //   (A's function table + all incoming messages - all outgoing messages)
    // - multiply all entries of theta by 1 / A->weight_forward
    // - for every incoming edge e, if e is_forward, sub_assign theta from e's message

    // backward pass

	// set LB = LB_init;
    // for every factor A in reverse order, do the following:
    // - for every incoming edge e that either is_forward or is_update_lb,
    //   - let v = result of send message along e
    //   - if is_update_lb, set LB += v
    // - if compute_solution, compute solution
    // - let theta be the current reparametrization of A
    //   (A's function table + all incoming messages - all outgoing messages)
    // - multiply all entries of theta by 1 / A->weight_backward
    // - if A->compute_bound and A->weight_backward > 0
    //   - let A_weight = A->weight_backward - #(incoming edges for A that is_backward) [move to NodeEdgeAttrs init]
    //   - set LB += theta.min() * A_weight
    // - for every incoming edge e, if e is_backward, sub_assign theta from e's message
