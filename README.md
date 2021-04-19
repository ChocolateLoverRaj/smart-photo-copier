# smart-photo-copier
Copy folders while avoiding duplicates.

## Caveats
Things to be careful of when using this tool.

### Duplicate photos in source dir
The program only checks the exist dir for duplicate photos, not the source dir. This means that if there are two identical files in the source dir, they can both get copied.

### Multiple files with same name in exist dir
When checking the compare file contents option, only one of the source files with the duplicate names are checked. This can lead to copying files unnecessarily.

Before
```
- exist
  - a
    - file.txt (contents: 'hi')
  - b
    - file.txt (contents: 'hello')
- src
  - file.txt (contents: 'hello')
```

After
```
- exist
  - a
    - file.txt (contents: 'hi')
  - b
    - file.txt (contents: 'hello')
  - file.txt (contents: 'hello')
- src
  - file.txt (contents: 'hello')
```

This happens because `exist/a/file.txt` (`hi`) is being compared with `src/file.txt` (`hello`). Since `hi` and `hello` are different the file gets copied, even though `exist/b/file.txt` already exists with the same contents.