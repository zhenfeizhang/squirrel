#ifndef POLY_H
#define POLY_H
#include <stdint.h>
#include <stdlib.h>
#include "params.h"

// a naive implementation of ternary polynomial multiplication
// inputs:
// - a: binary polynomial of length N
// - b: ternary polynomial with 22 non-zero, sorted coefficients
// output:
// - c: polynomial of length N
void ter_poly_mul(int8_t *c, const int8_t *a, const uint8_t *b_index,
                  const uint8_t *b_sign);

int ternary_mul(
    int8_t *res,     /* out - a * b % (x^N+1) in Z[x], must be length N */
    uint8_t *buf,    /*  in - scratch space of size 4.5N */
    int8_t const *a, /*  in - polynomial a */
    uint8_t const *b_indices); /*  in - polynomial b's indices */


/// convert a polynomial into its NTT form
void hvc_ntt(uint16_t p[N]);

/// convert an NTT form polynomial into its integer form
void hvc_inv_ntt(uint16_t p[N]);   


/// convert a polynomial into its NTT form
void hots_ntt(uint32_t p[N]);

/// convert an NTT form polynomial into its integer form
void hots_inv_ntt(uint32_t p[N]);   

#endif