## Markdown

# Tuple

# Record Ids:

- dbms assigs each logical tuple a unique record identifier that represents its physical location in db, usually it contains:
    1. file Id
    2. page Id
    3. slot
- SQLite uses ROWID as the true primary key and stores them as a hidden attribute  (6 bytes to 10 bytes)

- size of this defines the size of are page/slots 

# Tuple header
- each tuple is prefixed with a header that contains meta data about it
   1. visibility info for concurrency control 
   2. bit map for null value 


# Tuple-oriented-storage problem:
 - fragmentation - page are not fully utilized
 - useless Disk I/O - must fetch entire page to update one tuple
 - random Disk I/0 , worst case, every tuple we are updating is on separate page 
 
## Log structured storage(RocksDB...):
 - instead of storing tuples in pages, the DBMS maintains a log that record changes to tuple
 - each log represents PUT/DELETE operation
 - page is not-volatile memory is immutable
 - to make reed quicker you need to have pointer array which will point to latest PUT/DELETE for tuple
 - periodicy clean up log pages, so only latest logs are saved (expensive operation)
 
 - universal compaction - takes 2 files and compact them into one, and then repeat
 - level compaction - each level has 2 pages, when they are filled ,they are compacted and send to level below, then repeat 
 - downside of using this approach is write amplification when compacting(writing page in storage, then taking it out to compact it, then writing again)
 
### Index organized storage(SQLite, MySQL, SQLServer...)
- approaches before need to have some array to keep info about where is the actual data(because it isn't structured in memory as its indexed)
- here we use index data structure
- use a page layout that looks like slotted page(where tuples are typic ally sorted in page based on key)

## Word alignment 
 - you want to contain individual data(INT,TIMESTAMP) to a word(let say 64 bits), so you will need to pad with 0s if some word doesn't fit in it.
 
 e.g. INT(32), TIMESTAMP(64), CHAR(2), INT(32)
 
 you would place this as:
 32 + pad | 64 | 2 + 32 + pad 
 
 notice you will padded int to make timestamp in one word, for continues bytes(e.g. strings) you place pointer to an address
  

## Data reporesentaion:

1. INT, BIGINT, SMALLINT, TINYINT - same as C
2. FLOAT,REAL vs NUMERIC, DECIMAL - IEEE-754 Standard / Fixed point Decimal
3. VARCHAR, VARBINARY/ TEXT/ BLOB - header with length, folowed by bytes OR pointer to another page / offset with data [if bigger then word]
4. TIME/DATE/TIMESTAMP/INTERVAL - 32/64 bit integer

# Null data type 
 Choice:
   1. store a bitmap in a centralized header that specifies which attributes are null(most common approach)
   2. special value - designate a value to represent NULL for a data type  (e.g. INT32_MIN)
   3. per attribute null flag - stores a flag that marks that a value in null(use more space because of the word alignment)


### Buffer pool manager
 - memory region organized as an array of fixed-size pages (array entity is called frame)
 - dirty pages are buffered and not written to disk immediately (write-back cash)
 - buffer pool has meta data that
     1. keep track what pages are currently in memory (usually a fixed-size hash table protected with latches to ensure thread-safe access)
     2. dirty flag
     3. access tracking info
     4. pin/reference counter (number of workers that require this page to stay in memory)
 - page table: mapping from page ids to a copy of a page in buffer pool frames
 - page directory: mapping from page ids to page location in the db files (must be persistant)
 
## Buffer pool optimization
 
# Multiple buffer pools:
 - DBMS does not always have a single buffer pool for the entire system thre can be:
    1. multiple buffer pool instance
    2. per-database buffer pool
    3. per-page type buffer pool
 - this is done to help reduce latch contention and improve locality
 
 
## Hash 

# Hash functions

 - we want something that fast and has low collision rate
 - e.g. of usage in db is FarmHash and XXHash(best in the time of writing)

## Static hashing schemes:
 - approachs:
     1. linear probe hashing
     2. cuckoo hashing
     3. Roobin hood hashing
     4. Hopscoth hashing
     5. Swiss table
     
     
 - require to know number of elemens we want to store, if we accede that limit we need to make that data structure again...(unlike dynamic hashing)
     
# Lienar probe hahing 
   - array of slots
   - if colliion happens then insert data to next slot (slot has 2 values, its key and data, so we now if we are looking at wrong slot)
   - when deleteing data from the slots we can get a problem if collision prevention accrued, so when we delete we must say if something was there, so we set special value if we are deleting slot
     
# Cuckoo hashing
- use multiple hash functions to find multiple locations when inserting
- on insert, pick the location that is empty and save entry to hash table
- if no location is available, evict the element from one of them and then re-hash it to find a new location

- look up and deletions : O(1)

## Dynamic hash

# Chained hashing
- pointers in hash table point to linked list
- to optimize use bloom filter(can have false negative, but cant have false positives)


## B+ tree

- self balancing, ordered tree data structure that allows searches, sequential access, insertion, and deletion in O(log n) 
- optimize for system that read and write larger block of data

- properties:
  1. perfectly balanced (every leaf node is at the same depth in the tree)
  2. every node other than the root is at least half-full
  3. every inner node with k keys has k+1 non-null children

- every leaf has sibling pointers

- insert:
  1. find correct left node
  2. insert data entry into L in sorted order
  
  if there is no enough space in L, split L keys into L and a new node L2
    1. redistribute entries evenly, copy up middle key
    2. insert index entry pointing to L2 into parent of 
  to split inner node, redistribute entire evenly but push up middle key
    
  
 - delete
  1. remove the entry in leaf node 
     if L i at least half-full done
     if L has only M/2 - 1 entries: try to distribute, borrowing from sibling, if re-distribution fails, merge L and sibling(+ update parent)
