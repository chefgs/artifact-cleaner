# Turn your executable into `brew install`: a Homebrew tap guide for Rust, Go, Java and Node

*You do not need Homebrew's approval to publish a tool. You need a public GitHub repo and one small Ruby file. This guide shows the same recipe for four ecosystems, and where each one differs.*

> **What is verified.** The Rust path is the one we actually shipped and tested (`artifact-cleaner`, see the companion article). The Go, Java and Node sections follow Homebrew's documented formula patterns, but we did not publish those four-ecosystem examples here, so test each one with `brew audit` and `brew install` before relying on it. Check the current docs of any tool named below (GoReleaser, `cargo-dist`) for exact config keys.

---

## 1. The mental model

A tap is a **price list**. It does not hold your software. It holds a note that says: *"for version X, download this file from this URL; its fingerprint is Y; copy this into `bin`."*

```
Your code → build → archive (.tar.gz) + fingerprint (sha256)   ← public URL
                                  ▲
Formula (.rb): url + sha256 + install steps ── points at ──┘
        │ lives in
        ▼
Tap: a public GitHub repo named  homebrew-<name>
```

When someone runs `brew install owner/name/tool`, Homebrew expands `owner/name` to the repo `owner/homebrew-name`, reads `Formula/tool.rb`, downloads the file, checks the sha256, and runs the formula's `install` method.

The tap is the same for every language. **What changes is what the formula downloads and what it needs on the user's machine.**

## 2. Pick your approach: prebuilt or build from source

| | **Prebuilt binary** | **Build from source** |
|---|---|---|
| The formula downloads | Your compiled archive | Your source tarball |
| User needs | Nothing extra | A compiler or runtime at install time |
| Install speed | Seconds | Slow (compiles on the user's machine) |
| You must build | One archive per OS/CPU | Nothing; Homebrew builds it |
| Best for | Rust, Go (static binaries) | Small tools, or when you do not want a release pipeline |

And for languages that need a runtime:

| Ecosystem | Natural fit | The formula's extra job |
|---|---|---|
| **Rust** | Prebuilt binary per platform | Pick the right archive for OS and CPU |
| **Go** | Prebuilt binary per platform | Same as Rust |
| **Java** | One platform-independent jar | Depend on a JDK and write a launcher script |
| **Node** | The package from the npm registry | Depend on `node` and install with npm |

If you can produce a single self-contained executable (Rust, Go, or Java/Node compiled to native), treat it like Rust or Go.

## 3. Steps that are the same everywhere

### 3.1 Create the tap (once)

```sh
gh repo create OWNER/homebrew-tap --public --description "Homebrew formulae"
git clone git@github.com:OWNER/homebrew-tap.git
mkdir -p homebrew-tap/Formula
```

The `homebrew-` prefix is mandatory and is dropped in the short name, so users type `OWNER/tap`. One tap holds any number of tools, one file each in `Formula/`.

### 3.2 The file name decides the class name

`Formula/my-tool.rb` must contain `class MyTool < Formula` (CamelCase of the file name). Do not clash with an existing core formula name, or users get "which one?" ambiguity.

### 3.3 Get a checksum

For any downloadable file:

```sh
curl -fsSL <url> | shasum -a 256
```

Better, have your release pipeline write a `.sha256` file next to every archive, and let a script read it when it generates the formula. Never paste checksums by hand.

### 3.4 The URL must be public

Homebrew downloads anonymously. Release files of a **private** GitHub repo return 404, so for a private source repo, host the archives on the public tap's Releases instead. If your source repo is public, point the formula at its own Releases.

### 3.5 Always test with Homebrew's own tools

```sh
brew style Formula/my-tool.rb                     # fast, works on a file path
brew install OWNER/tap/my-tool
brew test OWNER/tap/my-tool
brew audit --strict --online OWNER/tap/my-tool
```

If Homebrew 7 reports the tap as untrusted, the user (or you) runs `brew trust OWNER/tap`. Mention it in your tap README.

### 3.6 Keep four things in agreement

**The git tag, the version in your manifest, the archive file names, and the formula URLs.** Almost every failed release is one of these drifting.

---

## 4. Rust (prebuilt binary)

Rust and Go are the easiest cases: a native binary with no runtime.

### Build

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --locked --release --target aarch64-apple-darwin
cargo build --locked --release --target x86_64-apple-darwin
```

- On Apple Silicon you can cross-compile the Intel macOS target.
- For Linux, prefer the `-musl` targets (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`) for fully static binaries that run on any distro.
- `--locked` fails the build if `Cargo.lock` is out of date. Commit `Cargo.lock`.

### Package and checksum (in CI, on a tag)

```sh
name="mytool-v${VERSION}-${TARGET}"
mkdir "$name" && cp target/${TARGET}/release/mytool LICENSE README.md "$name/"
tar -czf "$name.tar.gz" "$name"
shasum -a 256 "$name.tar.gz" > "$name.tar.gz.sha256"
```

Attach both files to a GitHub Release (`gh release create` or `gh release upload`). Pick **one** naming pattern and never change it; the formula depends on it.

### Formula

```ruby
class Mytool < Formula
  desc "Short description under 80 characters"
  homepage "https://github.com/OWNER/mytool"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/OWNER/mytool/releases/download/v1.0.0/mytool-v1.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "…"
    end
    on_intel do
      url "https://github.com/OWNER/mytool/releases/download/v1.0.0/mytool-v1.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "…"
    end
  end

  on_linux do
    on_arm do
      url "…/mytool-v1.0.0-aarch64-unknown-linux-musl.tar.gz"
      sha256 "…"
    end
    on_intel do
      url "…/mytool-v1.0.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "…"
    end
  end

  def install
    bin.install "mytool"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mytool --version")
  end
end
```

Notes from the real publish:

- Homebrew strips a single top-level folder inside the archive, so `bin.install "mytool"` works with a bare name.
- The version is read from the file name. Do **not** add a `version` line; `brew audit` flags it as redundant.
- If the crate ships two binaries (a long name and a short alias), add a second `bin.install`.
- `desc` must be under 80 characters or `brew style` fails.

### Alternative: build from source

If you have no release pipeline, the formula can compile on the user's machine:

```ruby
url "https://github.com/OWNER/mytool/archive/refs/tags/v1.0.0.tar.gz"
sha256 "…"
depends_on "rust" => :build

def install
  system "cargo", "install", *std_cargo_args
end
```

Simple to publish, slower to install.

### Tooling

`cargo-dist` can generate the cross-platform release workflow and a formula from small config. Worth evaluating once you maintain several tools. Read its current docs before adopting.

---

## 5. Go (prebuilt binary)

Same shape as Rust. Go cross-compiles trivially.

### Build

```sh
CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -ldflags "-s -w" -o mytool .
CGO_ENABLED=0 GOOS=darwin GOARCH=amd64 go build -ldflags "-s -w" -o mytool .
CGO_ENABLED=0 GOOS=linux  GOARCH=amd64 go build -ldflags "-s -w" -o mytool .
CGO_ENABLED=0 GOOS=linux  GOARCH=arm64 go build -ldflags "-s -w" -o mytool .
```

- `CGO_ENABLED=0` gives a static binary with no C library dependency.
- `-s -w` strips debug info and shrinks the binary.
- Inject the version at build time so `--version` is correct: `-ldflags "-X main.version=1.0.0"`.

### Package, release and formula

Packaging, checksums and the formula are identical to the Rust section: a `.tar.gz` plus `.sha256` per `GOOS/GOARCH`, and `on_macos` / `on_linux` blocks with `on_arm` / `on_intel`. Go's names for the CPUs are `arm64` and `amd64`, so map them in your archive names consistently.

### Tooling: GoReleaser

GoReleaser is the Go ecosystem's standard for this. From one config it builds every platform, makes archives and checksums, creates the GitHub Release, and can generate and push a Homebrew formula to your tap. It needs a token with write access to the tap repo, stored as a CI secret. Its config key for Homebrew has changed across versions, so follow its current documentation rather than copying an old example.

### Alternative: build from source

```ruby
url "https://github.com/OWNER/mytool/archive/refs/tags/v1.0.0.tar.gz"
sha256 "…"
depends_on "go" => :build

def install
  system "go", "build", *std_go_args(ldflags: "-s -w")
end
```

---

## 6. Java (a jar plus a JDK)

A Java tool is usually one **platform-independent jar**, so there is a single download and no per-CPU blocks. The catch is that the user needs a JVM, and the formula must provide a launcher script.

### Build

Produce a runnable ("fat" or "uber") jar, so all dependencies are inside one file:

```sh
mvn -B package          # Maven (use the shade or assembly plugin)
./gradlew shadowJar     # Gradle (Shadow plugin)
```

Attach `mytool-1.0.0.jar` to a GitHub Release and write a checksum:

```sh
shasum -a 256 mytool-1.0.0.jar > mytool-1.0.0.jar.sha256
```

### Formula

```ruby
class Mytool < Formula
  desc "Short description under 80 characters"
  homepage "https://github.com/OWNER/mytool"
  url "https://github.com/OWNER/mytool/releases/download/v1.0.0/mytool-1.0.0.jar"
  sha256 "…"
  license "Apache-2.0"

  depends_on "openjdk"

  def install
    libexec.install "mytool-1.0.0.jar"
    bin.write_jar_script libexec/"mytool-1.0.0.jar", "mytool"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mytool --version")
  end
end
```

How it works:

- `depends_on "openjdk"` makes Homebrew install a JDK first. Use a versioned formula such as `openjdk@21` if your tool needs a specific Java version.
- `libexec` is where non-binary files go. You do not put a jar directly in `bin`.
- `write_jar_script` generates a small shell wrapper named `mytool` in `bin` that runs `java -jar` on the jar, using the Homebrew-managed JDK.

### Native alternatives (no JVM for the user)

- **GraalVM `native-image`** compiles your Java app into a native executable. You then publish per-platform archives and use the Rust/Go-style formula. You must build on each target OS (or use CI runners for each).
- **`jlink` / `jpackage`** can bundle a minimal runtime with your app, at the cost of larger downloads.

Use these only if requiring a JDK is a real adoption problem.

---

## 7. Node (an npm package)

For Node, the most natural source is the **tarball your package already has on the npm registry**. You publish to npm as usual, and the formula installs from it.

### Publish the package

Make sure `package.json` has a `bin` entry:

```json
{ "name": "mytool", "version": "1.0.0", "bin": { "mytool": "cli.js" } }
```

and that `cli.js` starts with `#!/usr/bin/env node`. Publish with `npm publish`.

### Get the tarball URL and checksum

```sh
npm view mytool@1.0.0 dist.tarball
curl -fsSL "$(npm view mytool@1.0.0 dist.tarball)" | shasum -a 256
```

(npm lists its own integrity hash in sha512; Homebrew wants sha256, so compute it yourself as above.)

### Formula

```ruby
class Mytool < Formula
  desc "Short description under 80 characters"
  homepage "https://github.com/OWNER/mytool"
  url "https://registry.npmjs.org/mytool/-/mytool-1.0.0.tgz"
  sha256 "…"
  license "MIT"

  depends_on "node"

  def install
    system "npm", "install", *std_npm_args
    bin.install_symlink Dir["#{libexec}/bin/*"]
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mytool --version")
  end
end
```

How it works:

- `depends_on "node"` installs Node first. Use `node@22` or similar if you need a specific major version.
- `std_npm_args` installs the package and its dependencies into `libexec`, isolated from the user's global npm.
- `bin.install_symlink` links the package's executables into Homebrew's `bin`.

### Native alternative: a single executable

If you do not want users to need Node, build a standalone executable (Node's built-in Single Executable Applications feature, or `bun build --compile`), then ship per-platform archives and use the Rust/Go-style formula. Check each tool's current docs for supported platforms and limits.

---

## 8. Automating the tap

Everything above can be done by hand. When you have a few tools, automate:

1. **A generator script** that reads the release's `.sha256` files and writes the formula (write to a temp file and `mv`, so a failure never leaves an empty formula).
2. **A release workflow** triggered by `v*` tags that builds, packages, checksums and creates the GitHub Release.
3. **A tap-update step** that commits the formula. It needs a token with write access to the tap repo, stored as a repository secret. Keep CI permissions read-only except for the job that needs to write.

## 9. Pitfalls that apply to every language

| Symptom | Likely cause |
|---|---|
| `404` downloading a release file that exists | Source repo is private; host archives publicly |
| Checksum mismatch | The archive was rebuilt or replaced after the formula was generated. Never rebuild a published release; cut a new version |
| `brew audit`: redundant `version` | Remove it; Homebrew reads it from the URL |
| `brew style`: description too long | Keep `desc` under 80 characters |
| Tag push builds nothing | A conditional job upstream is skipped, and its dependents silently skip too. Use `!cancelled()` and check results explicitly |
| Release "succeeded" but a file is missing | Skipped jobs do not fail a run. Count the assets |
| Upgraded, but the old version still runs | Another copy earlier on `PATH`. Run `which -a <tool>` |
| "Untrusted tap" warning | `brew trust OWNER/tap` |

## 10. One-page checklist

1. Create the public `homebrew-tap` repo with a `Formula/` folder (once).
2. Choose the approach: prebuilt binary, jar plus JDK, npm package, or build from source.
3. Make the release produce the downloadable file and a sha256 for every version.
4. Write (or generate) `Formula/<tool>.rb`, with the class name matching the file name.
5. `brew style`, then `brew install`, `brew test`, `brew audit --strict --online`.
6. Add the install line and the `brew trust` note to the tap README.
7. For each new version: tag, build, regenerate the formula, push the tap, and verify with `brew upgrade` and `which -a`.

The same four numbers must always agree: **tag, manifest version, file names, formula URLs.**
