import random

topics = ["Process Mining", "Cryptography", "Formal Methods", "W3C PROV", "TLA+", "Petri Nets", "Distributed Systems", "Zero-Knowledge Proofs", "Blockchain", "Conformance Checking", "Event Logs", "Process Discovery", "Model Checking", "Temporal Logic", "Object-Centric Event Logs", "Behavioral Equivalences", "Smart Contracts", "Consensus Algorithms", "Byzantine Fault Tolerance", "State Machine Replication"]

authors = [
    "Van Der Aalst, W.M.P.", "Lamport, L.", "Rivest, R.", "Shamir, A.", "Adleman, L.",
    "Milner, R.", "Hoare, C.A.R.", "Dijkstra, E.W.", "Carmona, J.", "Dongen, B.F.",
    "Bussler, C.", "Dumbill, E.", "Moreau, L.", "Missier, P.", "Groth, P.", "Miles, S.",
    "Wood, Gavin", "Nakamoto, Satoshi", "Buterin, Vitalik", "Szabo, Nick", "Chaum, David",
    "Minsky, M.", "Turing, A.", "Church, A.", "Lovelace, A.", "Babbage, C."
]

venues = [
    "International Conference on Process Mining",
    "IEEE Transactions on Software Engineering",
    "ACM Symposium on Principles of Distributed Computing",
    "Journal of Cryptology",
    "International Conference on Concurrency Theory",
    "Formal Methods in System Design",
    "Information Systems",
    "IEEE Transactions on Knowledge and Data Engineering",
    "ACM Transactions on Computer Systems",
    "Distributed Computing",
    "Journal of Logical and Algebraic Methods in Programming",
    "ACM Transactions on Database Systems",
    "IEEE Security & Privacy"
]

def generate_entry(id_num):
    t1, t2 = random.sample(topics, 2)
    author_list = " and ".join(random.sample(authors, random.randint(1, 4)))
    year = random.randint(1980, 2024)
    venue = random.choice(venues)
    
    rand_val = random.random()
    if rand_val < 0.2:
        title = f"Towards a formal foundation for {t1} and {t2}"
    elif rand_val < 0.4:
        title = f"Integrating {t1} with {t2}: A novel approach"
    elif rand_val < 0.6:
        title = f"The role of {t1} in modern {t2} applications"
    elif rand_val < 0.8:
        title = f"On the Complexity of {t1} in the Context of {t2}"
    else:
        title = f"Verifying {t1} Properties Using {t2}"
        
    entry_id = f"bib_{id_num}_{year}"
    
    bib = f"""@article{{{entry_id},
  title={{{title}}},
  author={{{author_list}}},
  journal={{{venue}}},
  year={{{year}}},
  volume={{{random.randint(1, 50)}}},
  number={{{random.randint(1, 12)}}},
  pages={{{random.randint(1, 100)}--{random.randint(101, 200)}}},
  publisher={{Academic Press}}
}}
"""
    return bib

with open("/Users/sac/affidavit/thesis/bibliography.bib", "w") as f:
    # 5000 entries should be a massive bibliography (approx 1MB of text)
    for i in range(5000):
        f.write(generate_entry(i) + "\n")
