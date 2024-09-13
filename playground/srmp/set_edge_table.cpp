#include <string.h>
#include <iostream>

void SetEdgeTable_orig(int nA, int* A, int* AK, int nB, int* B, int* BK, int* table) {
	int i, j, b=0, k=0;
	int K_array[nB];
	int labeling[nB];
	for (i=0; i<nB; i++)
	{
		K_array[i] = 1;
		for (j=nA-1; A[j] != B[i]; j--) K_array[i] *= AK[j];
	}
	memset(labeling, 0, nB*sizeof(int));
	table[0] = 0;
	while ( 1 )
	{
		for (i=nB-1; i>=0; i--)
		{
			if (labeling[i] < BK[i] - 1) break;
			k -= labeling[i]*K_array[i];
			labeling[i] = 0;
		}
		if (i<0) break;
		labeling[i] ++;
		k += K_array[i];
		table[++b] = k;
	}
}

void SetEdgeTable_edit(int nA, int* A, int* AK, int nB, int* B, int* BK, int* table) {
    // Args:
    // - nA: number of variables in factor A
    // - A: list of variable identifiers in A (sorted in increasing order)
    // - AK: list of dimensions of variables in A
    // - nB: number of variables in factor B
    // - B: list of variable identifiers in B (sorted in increasing order)
    // - BK: list of dimensions of variables in B
    // - table: stride table to fill in (size = product of everything in BK, aka size of function table of B)

	int K_array[nB];
	for (int i = 0; i < nB; i++)
	{
		K_array[i] = 1;
		for (int j = nA - 1; A[j] != B[i]; j--) {
            K_array[i] *= AK[j];
        }
	}

	int labeling_of_B[nB];
	memset(labeling_of_B, 0, nB * sizeof(int));

	table[0] = 0;
    int i = nB - 1;
	int b = 0;
    int k = 0;
    while(true) {
        if (labeling_of_B[i] < BK[i] - 1) {
            // Move to next variable label
            labeling_of_B[i]++;
            k += K_array[i];
            b++;
            table[b] = k;
            i = nB - 1;
        } else {
            // "Carry"
            k -= labeling_of_B[i] * K_array[i];
            labeling_of_B[i] = 0;
            i--;
            if (i < 0) {
                break;
            }
        }
    }
}

void print_array(int size, int* array) {
    for (int i = 0; i < size; i++) {
        std::cout << array[i] << ' ';
    }
    std::cout << '\n';
}

void compare_arrays(int size, int* array1, int* array2) {
    for (int i = 0; i < size; i++) {
        if (array1[i] != array2[i]) {
            std::cout << "------------------------------ Arrays mismatch at position" << i << " ------------------------------\n";
        }
    }
}

int product(int n, int* array) {
    int result = 1;
    for (int i = 0; i < n; i++) {
        result *= array[i];
    }
    return result;
}

void get_domain_sizes(int num_variables, int* domain_sizes, int factor_size, int* factor_variables, int* factor_domains) {
    for (int i = 0; i < factor_size; i++) {
        factor_domains[i] = domain_sizes[factor_variables[i]];
    }
}

void get_variable_difference(int nA, int* A, int nB, int* B, int* C) {
    int i = 0;
    int j = 0;

    for (; ; i++) {
        while (j < nB && A[i] == B[j]) {
            i++;
            j++;
        }
        if (j == nB) {
            break;
        }
        C[i - j] = A[i];
    }
	for ( ; i < nA; i++) {
        C[i - j] = A[i];
    }
}

int main() {
    int num_variables = 3;
    int domain_sizes[num_variables] = {3, 4, 5};

    // First factor
    int nA = 3;
    int A[nA] = {0, 1, 2};  // must be sorted
    int AK[nA];
    get_domain_sizes(num_variables, domain_sizes, nA, A, AK);

    // Second factor, subset of first factor
    int nB = 1;
    int B[nB] = {1};  // must be sorted and subset of A
    int BK[nB];
    get_domain_sizes(num_variables, domain_sizes, nB, B, BK);

    std::cout << "Num variables: " << num_variables << '\n';
    std::cout << "Domain sizes: ";
    print_array(num_variables, domain_sizes);
    std::cout << "Alpha variables: ";
    print_array(nA, A);
    std::cout << "Beta variables: ";
    print_array(nB, B);

    // First table
    int first_table_size = product(nB, BK);

    // Using original implementation
    int first_table_orig[first_table_size];
    SetEdgeTable_orig(nA, A, AK, nB, B, BK, first_table_orig);
    std::cout << "First table: ";
    print_array(first_table_size, first_table_orig);

    // Using edited implementation
    int first_table_edit[first_table_size];
    SetEdgeTable_edit(nA, A, AK, nB, B, BK, first_table_edit);
    compare_arrays(first_table_size, first_table_orig, first_table_edit);

    // Difference between A and B
    int nC = nA - nB;
    int C[nC];
    get_variable_difference(nA, A, nB, B, C);
    int CK[nC];
    get_domain_sizes(num_variables, domain_sizes, nC, C, CK);

    // Second table
    int second_table_size = product(nA, AK) / first_table_size;

    // Using original implementation
    int second_table_orig[second_table_size];
    SetEdgeTable_orig(nA, A, AK, nC, C, CK, second_table_orig);
    std::cout << "Second table: ";
    print_array(second_table_size, second_table_orig);

    // Using edited implementation
    int second_table_edit[second_table_size];
    SetEdgeTable_edit(nA, A, AK, nC, C, CK, second_table_edit);
    compare_arrays(first_table_size, first_table_orig, first_table_edit);

    return 0;
}