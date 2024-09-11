    // SRMP forward pass
    for (t=0; t<seq.num; t++)
    {
        // step 4
        for (e=B->first_in; e; e=e->next_in) {
            if (e->is_bw) {
                SEND_MESSAGE(e);
            }
        }

        // step 5
        B = seq.arr[t].A;
        int b, K = B->K;
        if (B->arity == 1) memcpy(theta, B->data, K*sizeof(double));  // unary => copy function table for beta from CFN, no outgoing edges
        else COMPUTE_PARTIAL_REPARAMETERIZATION((NonSingletonFactor*)B, theta);  // copy function table from CFN (or create 0 array if it doesn't exist), subtract outgoing edges
        for (e=B->first_in; e; e=e->next_in) {
            for (b=0; b<K; b++) theta[b] += e->m[b];  // add incoming edges
        }

        // step 6
        double p = 1.0 / B->weight_forward;
        for (b=0; b<K; b++) theta[b] *= p;
        for (e=B->first_in; e; e=e->next_in)
        {
            for (b=0; b<K; b++) e->m[b] -= theta[b];  // in this implementation, e->weight_forward = 1 always
        }
    }



    // SRMP init fragment (weight calculations)
	int default_weight = 1;
	if (options.TRWS_weighting != (double)((int)options.TRWS_weighting)) default_weight = 10;
    int w_forward_out = 0, w_backward_out = 0;
    int w, w_forward_in = 0, w_backward_in = 0, w_total_in = 0;
    for (e=A->first_in; e; e=e->next_in)
    {
        int e_weight = default_weight; // can be changed to other values (perhaps, dependent on arities?)

        if (e->is_fw)
        {
            e->weight_forward = e_weight;
            w_forward_in += e_weight;
        }
        else e->weight_forward = 0;

        if (e->is_bw)
        {
            e->weight_backward = e_weight;
            w_backward_in += e_weight;
        }
        else e->weight_backward = 0;

        w_total_in += e_weight;
    }

    // if TRWS_weighting == 0 then A->weight_forward = w_forward_out + w_forward_in
    // if TRWS_weighting == 1 then A->weight_forward = w_forward_out + max ( w_forward_in, w_total_in-w_forward_in )

    int delta_forward = (w_total_in - w_forward_in) - w_forward_in; if (delta_forward < 0) delta_forward = 0;
    //int delta_forward = w_backward_in - w_forward_in; if (delta_forward < 0) delta_forward = 0;
    delta_forward = (int)(options.TRWS_weighting*delta_forward);
    w = w_forward_out + w_forward_in + delta_forward;
    A->weight_forward = (unsigned)w;
    if ((int)A->weight_forward != w) { printf("Error: capacity of Factor::weight_forward is not enough!\n"); exit(1); }

    // similarly for backward

    int delta_backward = (w_total_in - w_backward_in) - w_backward_in; if (delta_backward < 0) delta_backward = 0;
    //int delta_backward = w_forward_in - w_backward_in; if (delta_backward < 0) delta_backward = 0;
    delta_backward = (int)(options.TRWS_weighting*delta_backward);
    w = w_backward_out + delta_backward + w_backward_in;
    A->weight_backward = (unsigned)w;
    if ((int)A->weight_backward != w) { printf("Error: capacity of Factor::weight_backward is not enough!\n"); exit(1); }



double GeneralFactorType::SendMPLPMessages(Energy::NonSingletonFactor* A, bool set_solution)
{
	int total_weight = A->weight_forward;
	int a, b, c, A->K;
	double delta = 0;

	Energy::Edge* e;

	double* theta = (double*) rbuf.Alloc(A->K*sizeof(double) + 4*A->arity*sizeof(int));
	if (A->data) memcpy(theta, A->data, A->K*sizeof(double));
	else         memset(theta, 0, A->K*sizeof(double));

	for (e=A->first_in; e; e=e->next_in)
	{
		for (a=0; a<A->K; a++) theta[a] += e->m[a];
	}
	for (e=A->first_out; e; e=e->next_out)
	{
		int KB = e->B->K;
		int KC = A->K / KB;
		int* TB = (int*) e->send_message_data;
		int* TC = TB + KB;
		for (b=0; b<KB; b++)
		for (c=0; c<KC; c++)
		{
			theta[TB[b] + TC[c]] += e->B->rep[b];
		}
		total_weight += e->weight_forward;
	}

	if (set_solution) A->ComputeRestrictedMinimum(theta, (int*)(theta+A->K));

    delta = theta.max();
	// delta = theta[0];
	// for (a=1; a<A->K; a++)
	// {
	// 	if (delta > theta[a]) delta = theta[a];
	// }

	double total_weight_inv = 1.0 / total_weight;

	if (A->rep) memcpy(A->rep, theta, A->K*sizeof(double));

	for (e=A->first_out; e; e=e->next_out)
	{
		double rho = e->weight_forward * total_weight_inv;
		int KB = e->B->K;
		int KC = A->K / KB;
		int* TB = (int*) e->send_message_data;
		int* TC = TB + KB;
		for (b=0; b<KB; b++)
		{
			double v_min = theta[TB[b]]; // TC[c] == 0
			for (c=1; c<KC; c++)
			{
				if (v_min > theta[TB[b] + TC[c]]) v_min = theta[TB[b] + TC[c]];
			}
			e->B->rep[b] = rho * (v_min - delta);
		}
		if (A->rep)
		{
			for (b=0; b<KB; b++)
			{
				double v = e->B->rep[b];
				for (c=0; c<KC; c++)
				{
					A->rep[TB[b] + TC[c]] -= v;
				}
			}
		}
	}

	return delta;
}