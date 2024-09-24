    // ( initial lower bound )
	  // set LB_init = 0
    // for all non-unary factors A that have no incoming and no outgoing edges, do:
    // - LB_init += SEND_MPLP_MESSAGES(A); (without computing solution)

    // ( sorted factor sequence )
    // set seq.arr = unary factors + non-unary factors with at least one incoming edge
    // SortSequence(seq, options);

    // ( backward edges )
    // use A->tmp1 to mark if we encountered an edge pointing to factor A in forward pass (initially set to 0)
    // for every factor A in seq.arr, do the following:
    // - if A->tmp1 == 1, set A->compute_bound = 1 only if A is a unary factor (otherwise set it to 0)
    // - else           , set A->compute_bound = 1 and A->tmp1 = 1
    // - consider all incoming edges, let B be a source of such an edge
    //   - if B->tmp1 == 1, set e->is_bw = 1 and e->compute_bound = 0
    //   - else           , set e->is_bw = 0 and e->compute_bound = 1 and B->tmp1 = 1

    // ( forward edges )
    // use A->tmp2 to mark if we encountered an edge pointing to factor A in backward pass (initially set to 0)
    // for every factor A in seq.arr in reverse order, do the following:
    // - set A->tmp2 = 1
    // - consider all incoming edges, let B be a source of such an edge
    //   - if B->tmp2 == 1, set e->is_fw = 1
    //   - else           , set e->is_fw = 0 and B->tmp2 = 1

    // ( weights )
    // for every factor A in seq.arr, do the following:
    // - set w_forward_out = w_backward_out = 0;
    // - if A is non-unary, consider all outgoing edges, let B be a source of such an edge
    //   - if B comes after A in seq.arr, then w_forward_out += 1, else w_backward_out += 1
    // - set w_forward_in = w_backward_in = w_total_in = 0
    // - consider all incoming edges, let B be a source of such an edge
    //   - set w_total_in += 1
    //   - if e->is_fw, set e->weight_forward = 1 and w_forward_in += 1;
    //   - else       , set e->weight_forward = 0
    //   - if e->is_bw, set e->weight_backward = 1 and w_backward_in += 1;
    //   - else       , set e->weight_backward = 0
    // - compute A->weight_forward:
		// A->weight_forward = max(w_total_in - w_forward_in, w_forward_in) + w_forward_out
		// if (A->weight_forward + w_forward_in == 0) A->weight_forward = 1;
    // - compute A->weight_backward:
		// A->weight_backward = max(w_total_in - w_backward_in , w_backward_in) + w_backward_out
		// if (A->weight_backward + w_backward_in == 0) A->weight_backward = 1;

    // return LB_init
