## File comparison
Here is the method we will use to check if a file is exactly the same as another file, optimized for photos and videos:
- Check if the file size is exactly the same
- Read the first 4096 bytes of the two files
- Store the hash of the first 4096 bytes
- If the first 4096 bytes are the same, keep chunks of the next reading 4096 bytes until the end of the file 
- Wherever we end, also store the hash after the rest of the bytes
This is because we're mainly storing photos and videos. With photos, either they are actually the same file (100% identical), or just happen to have the same size (which in that case the first 4096 bytes will not be the same)

## Performance Optimizations
We can optimize for the following:
- Disk reads
- Total time

## Total time optimization
The issue with this is that we can't start reading thousands of files simultaneously. We *could*, but the computer can't physically read thousands of files at the same time. The operating system will have to have a queue for reading the files, or worse, it will round-robin between reading chunks of each file at a time, potentially being *slower* than if we limited simultaneous reads 1-32 files at once in our code.

### HDD
Hard drives can only read/write at one place at a time. They have best performance with sequential read/writes. So we should avoid jumping around and reading a little bit of one file and then a little bit of another file. Up to 32 operations can be queued.

### SATA SSD
Performance is worse when accessing spread out data compared to sequential, but not that bad. It can read/write to multiple places at once. Up to 32 operation can be queued.

### NVMe SSD
With this one we can do a bunch of operation simultaneously without worrying because NVMe's are able to read/write quickly in parallel and their queue can have up to 64,000 operations.

### SD Card and USB Flash Drive
These are kind of like HDDs where they have really bad random access performance. They also have really small queues.

## Reading directories recursively and reading metada
We are not going to limit simultaneous `read_dir`s or metadata access, we will just do them all at the same time. Hopefully the OS will optimize this.

## Copying files
If it's a SSD, we will copy multiple files at the same time. If it's something else, we will only copy 1 file at a time.
