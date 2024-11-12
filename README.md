# rust-change-detection
Change detection library written is rust, using file hashes to detect changes in files and directories.
Using `gxhash` for hashing files

## Usage

1. Install the package
```bash
yarn add rust-change-detection --dev
```
or

```bash
npm install rust-change-detection --save-dev
```

2. Set environment variable
```bash
export RCD_FOLDERS=./packages
```

3. Add rcd to your package.json scripts
```json
{
  "scripts": {
    "rcd": "rcd"
  }
}
```

4. Run the command
```bash
yarn rcd
```

5. You can find the logs in the `.rcd_log` file

## Tips

- You can create a `.env` file in the root of your project and add the `RCD_FOLDERS` variable there.
- And load the `.env` file before running the command, using a package like `dotenv-cli` or `cross-env` or `env-cmd`.

```json
{
  "scripts": {
    "rcd": "env-cmd .env rcd"
  }
}
```


## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|----------|----------|
| RCD_FOLDERS | The folders to watch for changes | none | true |
| RCD_IGNORE_FOLDER | The folders to ignore with `;` as separator | node_modules;dist;.git;coverage;.turbo | false |
| RCD_IGNORE_FILE | The files to ignore with `;` as separator | .gitignore;.prettierrc;.eslintrc;.babelrc;.DS_Store;Thumbs.db | false |
| RCD_HASH_FILE | The file to store the hashes | .rcd_hash | false |
| RCD_LOG_FILE | The file to store the logs, i.e the list of changed files | .rcd_log | false |
| RCD_LOG_LEVEL | The log level, can be `info` or `debug` | info | false |


## License
MIT
