#include "poly.h"
#include "params.h"
#include <immintrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// a naive implementation of ternary polynomial multiplication
// inputs:
// - a: binary polynomial of length N
// - b: ternary polynomial with 20 non-zero, sorted coefficients
// output:
// - c: polynomial of length N
void ter_poly_mul(int8_t *c, const int8_t *a, const uint8_t *b_index,
                  const uint8_t *b_sign) {
  unsigned int i, j;
  uint8_t r[2 * N];

  for (i = 0; i < 2 * N; i++)
    r[i] = 0;

  for (i = 0; i < 20; i++) {
    if (b_sign[i] == 1) {
      for (j = 0; j < N; j++) {
        r[j + b_index[i]] += a[j];
      }
    } else {
      for (j = 0; j < N; j++) {
        r[j + b_index[i]] -= a[j];
      }
    }
  }

  for (i = 0; i < N; i++)
    c[i] = r[i] - r[i + N];
  ;
}

// an AVX-2 implementation of ternary polynomial multiplication
// inputs:
// - a: binary polynomial of length N
// - b: ternary polynomial with 11 1s and 11 -1s
// output:
// - c: polynomial of length N
int ternary_mul(
    int8_t *res,     /* out - a * b % (x^N+1) in Z[x], must be length N */
    uint8_t *buf,    /*  in - scratch space of size 2N */
    int8_t const *a, /*  in - polynomial a */
    uint8_t const *b_indices) /*  in - polynomial b's indices */
{
  memset(buf, 0, (N << 1));

  __m256i tmp1;
  __m256i tmp2;
  __m256i base[16];

  for (int i = 0; i < 16; i++) {
    base[i] = _mm256_loadu_si256((__m256i *)(a + 32 * i));
  }

  for (int i = 0; i < 10; i++) {
    for (int j = 0; j < 16; j++) {
      tmp1 = _mm256_loadu_si256((__m256i *)(buf + 32 * j + b_indices[i]));
      tmp1 = _mm256_add_epi8(tmp1, base[j]);
      _mm256_storeu_si256((__m256i *)(buf + 32 * j + b_indices[i]), tmp1);

      tmp1 = _mm256_loadu_si256((__m256i *)(buf + 32 * j + b_indices[i + 10]));
      tmp1 = _mm256_sub_epi8(tmp1, base[j]);
      _mm256_storeu_si256((__m256i *)(buf + 32 * j + b_indices[i + 10]), tmp1);
    }
  }

  for (int i = 0; i < 16; i++) {
    tmp1 = _mm256_loadu_si256((__m256i *)(buf + i * 32));
    tmp2 = _mm256_loadu_si256((__m256i *)(buf + i * 32 + 512));
    tmp1 = _mm256_sub_epi8(tmp1, tmp2);
    _mm256_storeu_si256((__m256i *)(res + i * 32), tmp1);
  }

  return 0;
}