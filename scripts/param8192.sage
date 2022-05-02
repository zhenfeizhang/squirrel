reset()

# search for q
def search_q(beta, n):
    q = next_prime(beta * 4)
    while q % (2*n) != 1:
        q = next_prime(q)
    return q

# search for beta_s
def search_beta_s(n):
    for i in range(n):
        order = 2^i * factorial(n)/factorial(n-i)/factorial(i)
        if order > 2^256:
            return i

# search for alpha
def search_alpha(n):
    for i in range(n):
        order = 2^i * factorial(n)/factorial(n-i)/factorial(i)
        if order > 2^128:
            return i

# compute gamma
def compute_gamma(n, q):
    top = (3*128 + 1) + n * log(q, 2)
    bottom = (log(3, 2) - 1) * n
    gamma = RR(top/bottom)
    return gamma

# life time of a keypair
def get_life_time(tau):
    block_time = 10
    return RR(2^25*10/3600/24/364)

# search space of the randomizer
def get_search_space(n, alpha):
    return RR(log(factorial(n)/factorial(alpha)/factorial(n-alpha), 2)) + alpha

# size of a ring element
def ring_elem_size(n, logq):
    return n*logq

# hash cost
def hash_cost(logq, c1):
    return 2 * logq * c1


# number of validators
rho = 1024 * 8

# degree of the polynomial
n = 512

# number of non-zero entries in a challenge poly
# alpha is off by 1 so we have to reduce the security level a little bit
alpha = 20
print("alpha:", alpha, search_alpha(n))

# number of non-zero entries in a hash of message
beta_s = 44
log_beta_s = 6
print("beta:", beta_s, search_beta_s(n))

# norm bound for s1
# \beta_\sigma = 2\rho \alpha \beta_s
beta_sigma = 2 * rho * alpha * beta_s
log_beta_sigma = 21
print("beta_sigma:", beta_sigma, RR(log(beta_sigma, 2)))

# log of norm bound for aggregated sig
beta_agg = 8192
log_beta_agg = 12

# modulus q_hots in bits
q_hots = search_q(beta_sigma, n)
logq_hots = 26
print("q_hots: ", hex(q_hots), "which is ", RR(log(q_hots, 2)), "bits")

# modulus q_hvc in bits
q_hvc = search_q(beta_agg*15, n)
logq_hvc = 18
print("q_hvc: ", hex(q_hvc), "which is ", RR(log(q_hvc, 2)), "bits")

# number of elements in HOTS
gamma = compute_gamma(n, q_hots)
print("gamma: ", gamma)
gamma = 46

def print_sizes(tau, gamma, n, logq_hvc, log_beta_agg, log_beta_s):
    pk_size = ring_elem_size(n, logq_hvc)
    sig_size = ring_elem_size(n, logq_hvc) * (tau+2) + ring_elem_size(n, log_beta_s + 2) * gamma
    agg_sig_size = ring_elem_size(n, log_beta_agg + 1) * logq_hvc * 2 * (tau + 1) \
                 + ring_elem_size(n, log_beta_sigma +1 ) * gamma
    tree_size =  ring_elem_size(n, logq_hvc) * 2^(tau+1)

    print("pk size:", pk_size/8, "bytes")
    print("sig size:", sig_size/8/1024.0, "kilo bytes")
    print("aggregated sig size:", agg_sig_size/8/1024.0, "kilo bytes")
    print("tree size:", tree_size/8/1024/1024/1024.0, "giga bytes")

print("============================================")
for tau in [21, 24, 26]:
    print("tau:", tau)
    print_sizes(tau, gamma, n, logq_hvc, log_beta_agg, log_beta_s)
    # print_signing_cost(tau, gamma, n, logq, c1)
    # for h in [12, 16, 20]:
    #     print_signing_with_cache_cost(tau, n, logq, c1, h)
    # print()
    # print_key_gen_time(tau, gamma, logq, c1)
    # print_aggregation_time(tau, rho, logq, c1, c2)
    # print_verification_time(tau, gamma, logq, rho, c1)

    print("============================================")


# estimate the sis hardness

# tree
target = RR(sqrt(2 * n * log(q_hvc, 2)) * 4 * rho * alpha)
gaussian = RR(sqrt( (2 * logq_hvc  +1) * n/2/pi/e) * q_hvc^(1/(2 * logq_hvc + 1)))
print("tree sis gaussian:", (target / gaussian) ^ (1/(n*(2*logq_hvc +1))))

print("tree sis optimal:", RR(2^(2*sqrt(n * log(q_hvc) * log(1.005, 2)))), sqrt(target))

# host
target = RR(sqrt(gamma * n) * beta_sigma)
gaussian = RR(sqrt((gamma+1)*n/2/pi/e)*q_hots ^(1/(gamma+1)))
print("tree sis:", (target / gaussian) ^ (1/(n*(gamma +1))))


print("tree sis optimal:", RR(2^(2*sqrt(n * log(q_hots) * log(1.005, 2)))), sqrt(target))
