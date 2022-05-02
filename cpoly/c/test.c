#include "foreign/cpucycles.h"
#include "foreign/randombytes.h"
#include "foreign/speed.h"
#include "params.h"
#include "poly.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define NTESTS 1000

int test_ternary_mul() {
  unsigned int i, j;
  unsigned long long t1[NTESTS], t2[NTESTS], overhead;
  unsigned char seed[SEEDBYTES];
  int8_t *c1;
  int8_t *c2;
  int8_t *a;
  uint8_t *buf;
  uint8_t *b_index;
  uint8_t *b_sign;

  c1 = malloc(N);
  c2 = malloc(N);
  a = malloc(N);
  buf = malloc(2 * N);
  b_index = malloc(20);
  b_sign = malloc(20);

  overhead = cpucycles_overhead();
  randombytes(seed, sizeof(seed));

  for (i = 0; i < NTESTS; ++i) {
    // a is a random binary poly
    for (j = 0; j < N; j++) {
      a[j] = rand() % 2;
    }

    for (j = 0; j < 10; j++) {
      b_index[j] = rand() % 512;
      b_sign[j] = 1;
    }
    for (j = 0; j < 10; j++) {
      b_index[j + 10] = rand() % 512;
      b_sign[j + 10] = 0;
    }

    t1[i] = cpucycles_start();
    ter_poly_mul(c1, a, b_index, b_sign);
    t1[i] = cpucycles_stop() - t1[i] - overhead;

    t2[i] = cpucycles_start();
    ternary_mul(c2, buf, a, b_index);
    t2[i] = cpucycles_stop() - t2[i] - overhead;

    for (j = 0; j < N; j++) {
      if (c1[j] != c2[j]) {
        printf("%d-th FAILURE: c2[%u] = %u != %u\n", i, j, c1[j], c2[j]);
      }
    }
  }

  print_results("ternary: ", t1, NTESTS);
  print_results("ternary w simd: ", t2, NTESTS);

  free(c1);
  free(c2);
  free(a);
  free(buf);
  free(b_index);
  free(b_sign);

  return 0;
}

int test_hvc_ntt() {
  unsigned int i, j;
  unsigned long long t1[NTESTS], t2[NTESTS], overhead;
  unsigned char seed[SEEDBYTES];

  uint16_t *a;
  uint16_t *a_rec;

  a = (uint16_t*) malloc(N * sizeof(uint16_t));
  a_rec = (uint16_t*) malloc(N * sizeof(uint16_t));

  overhead = cpucycles_overhead();
  randombytes(seed, sizeof(seed));

  for (i = 0; i < NTESTS; ++i) {
    // a is a random poly
    for (j = 0; j < N; j++) {
      a[j] = rand() % 61441;
      a_rec[j] = a[j];
    }

    t1[i] = cpucycles_start();
    hvc_ntt(a_rec);
    t1[i] = cpucycles_stop() - t1[i] - overhead;

    t2[i] = cpucycles_start();
    hvc_inv_ntt(a_rec);
    t2[i] = cpucycles_stop() - t2[i] - overhead;
    
    for (j = 0; j < N; j++) {
      if (a[j] != a_rec[j]) {
        printf("%d-th FAILURE: a_rec[%u] = %u != %u\n", i, j, a[j], a_rec[j]);
      }
    }
  }

  print_results("hvc ntt: ", t1, NTESTS);
  print_results("hvc inv ntt: ", t2, NTESTS);

  free(a);
  free(a_rec);

  return 0;
}


int test_hots_ntt() {
  unsigned int i, j;
  unsigned long long t1[NTESTS], t2[NTESTS], overhead;
  unsigned char seed[SEEDBYTES];

  uint32_t *a;
  uint32_t *a_rec;

  a = (uint32_t*) malloc(N * sizeof(uint32_t));
  a_rec = (uint32_t*) malloc(N * sizeof(uint32_t));

  overhead = cpucycles_overhead();
  randombytes(seed, sizeof(seed));

  for (i = 0; i < NTESTS; ++i) {
    // a is a random poly
    for (j = 0; j < N; j++) {
      a[j] = rand() % 28930049;
      a_rec[j] = a[j];
    }

    t1[i] = cpucycles_start();
    hots_ntt(a_rec);
    t1[i] = cpucycles_stop() - t1[i] - overhead;

    t2[i] = cpucycles_start();
    hots_inv_ntt(a_rec);
    t2[i] = cpucycles_stop() - t2[i] - overhead;
    
    for (j = 0; j < N; j++) {
      if (a[j] != a_rec[j]) {
        printf("%d-th FAILURE: a_rec[%u] = %u != %u\n", i, j, a[j], a_rec[j]);
      }
    }
  }

  print_results("hots ntt: ", t1, NTESTS);
  print_results("hots inv ntt: ", t2, NTESTS);

  free(a);
  free(a_rec);

  return 0;
}

int main(void) {
  test_ternary_mul();
  test_hvc_ntt();
  test_hots_ntt();
  return 0;
}
