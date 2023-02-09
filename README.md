# zkdoc

> Data sharing privacy infrastructure for all

`zkdoc` is a project that aims to provide selective data sharing for documents via the use of zero knowledge proofs, specifically the Halo2 proof system. With the use of `zkdoc`, information within a document can be selectively shared by a user to a third party requestor without revealing additional data on the document that might not be required. 

## Crates & Application

This repository contains the complete source code of the zkdoc project.

- [`zkdoc_sdk`](./zkdoc_sdk)
- [`zkdoc_cli`](./zkdoc_cli)
- [`zkdoc_server`](./zkdoc_server) (Not published)


## Technical explanation of the underlying mechanism
We designed a protocol which involves zero knowledge proof that made the described scenario above possible. The issuer of the document will commit a hash of the document to the blockchain, utilizing blockchain as a settlement layer. The issuer could be the hospital, the university or auditors.
The document is then sent to the owner in plain text. The owner now is responsible of storing his/her own data. 
When sharing the public data to the recipient, the owner selected only the information that is shareable, creates a zero knowledge proof that the file hashes to the committed value on the blockchain and that the information indeed is part of the document.
The verifier upon receiving it, compares the commitment hash (public input) to that store on the blockchain and is able to accept (if proof true) or deny (if proof is false).
Our goal is to allow any party to have access to this technology and we created this project as an infrastructure, which allows the underlying zkp to be generic and would be flexible enough.

### Design of the document
A generic form that consist of two columns (one row for title, another row for content). Up to 10 rows.

eg: 
| Title | Content |
| --- | --- |
| Name | CC |
| AGe | xx |
|  |  |

Each row is able to store string of text. The title depends on the use case, for example in a health certificate, we can imagine that there will be row with the title `blood_type` with the `content` A+ for example. This can be as flexible as possible.


### Generation of commitment hash
We utilizes Poseidon hash function as it is a Snark friendly hash function that will allow us to reduce the number of constraints as compared to when we use sha256. 
We perform a double hashing mechanism (i.e horizontal and vertical).

#### Horizontal hashing
| Title | Content | Horizontal Result|
| --- | --- | --- |
| Name | CC |  hash_row_1 |
| AGe | xx | hash_row_2 |
| ... | ... |  ... |

The horizontal hashing hashes the title and content row by row, producing a resultant hash for that row as seen in `Horizontal Result` above.

#### Vertical hashing
| Horizontal Result|
| --- |
|  hash_row_1 |
| hash_row_2 |
|  ... |
| final accumulated |

The vertical hashing acts like an accumulator for the horizontal hashing. 
It takes the first two rows, hashes them together and results in the current accumulator. The current accumulator is hashed with the following row and returns the most recent accumulator result. The process repeats until all rows have been hashed.

The result of the accumulated hash will be our commitment hash that we will publish to the block chain.

By publishing on the commitment hash, we are:
1. ensuring that the file is not tampered with
2. maintain privacy to the content of the file

### Proof generation
The owner of document is able to select one row at a time to reveal (tbd to extend to multiple rows) and will generate a zk proof to proof that the information of the selected row is correct

The public inputs consist of the file commitment and the information of the selected row.

The private inputs will be the content of the entire document and also the `selector_index`, displayed as an array of ones and zeros. One representing the selected row.

### Concurrency management
As our solution is designed to be generic, there may be cases whereby the committed hash changes rather often. As a result, while the prover takes time to generate proof on a committed hash, a new hash overwrites the hash and this will cause the proof verification to fail.
As such, we design a ring buffer data structure to store the committed hash and the verifier is allowed to query the commited hash up to a certain history depth

### Circuit explanation
The private inputs is hashed and is constrained to the file commitment (public input). This ensure that the file is not tampered.

The selected row is checked by the circuit to ensure that it exist in the document. This is done by comparing the Horizontal hash with the Horizontal hash of our selected row in the public input

Finally, the selector index in also constraint to boolean values to ensure soundness of the proof.

