class GitHero < Formula
  desc "Fast and visual TUI for managing Git, written in Rust with Ratatui"
  homepage "https://github.com/MarlonRX/git-hero"
  url "https://github.com/MarlonRX/git-hero/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
  end

  test do
    assert_match "git-hero", shell_output("#{bin}/git-hero --version 2>&1 || true")
  end
end
