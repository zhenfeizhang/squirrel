

# - rev(i) is the reverse bit decomposition of i, i.e., 
#   0   ->  0
#   1   ->  100 0000
#   2   ->  010 0000 
#   3   ->  110 0000   ...
def reverse_bits(i, n):
    t = i.binary()[::-1]
    
    while len(t) < n:
        t = t + "0"

    res = 0    
    for e in t:
        res *= 2
        res += ZZ(e)
    return res

def print_hots_ntt():
    q_hots = 28930049
    while q_hots%4096!=1:
        q_hots = next_prime(q_hots)
    print(q_hots)

    P.<x> = PolynomialRing(Zmod(q_hots))
    f = P(x^1024+1)
    r = f.roots()[0][0]
    r_inv = 1/r
    print(r)

    for i in range (1024):
        e = reverse_bits(ZZ(i), 10)
        print(r^e, end = ', ')
        # print(i, e, r^e)
    print()

def print_hots_inv_ntt():
    q_hots = 28930049
    while q_hots%4096!=1:
        q_hots = next_prime(q_hots)

    P.<x> = PolynomialRing(Zmod(q_hots))
    f = P(x^1024+1)
    r = f.roots()[0][0]
    r_inv = 1/r
    print(r_inv)

    for i in range (1024):
        e = reverse_bits(ZZ(i), 10)
        print(r_inv^e, end = ', ')
    print()

def print_hvc_ntt():
    q_hvc = 61441
#    r = Zmod(q_hvc)(61)
    P.<x> = PolynomialRing(Zmod(q_hvc))
    f = P(x^1024+1)
    r = f.roots()[0][0]
    r_inv = 1/r
    print(r)

    for i in range (1024):
        e = reverse_bits(ZZ(i), 10)
        print(r^e, end = ', ')
        # print(i, e, r^e)
    print()

def print_hvc_inv_ntt():
    q_hvc = 61441
#    r = Zmod(q_hvc)(61)
    P.<x> = PolynomialRing(Zmod(q_hvc))
    f = P(x^1024+1)
    r = f.roots()[0][0]
    r_inv = 1/r
    print(r_inv)

    for i in range (1024):
        e = reverse_bits(ZZ(i), 10)
        print(r_inv^e, end = ', ')
    print()


print_hots_ntt()
print()
print_hots_inv_ntt()
print()
print_hvc_ntt()
print()
print_hvc_inv_ntt()

