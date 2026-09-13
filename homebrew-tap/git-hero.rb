class Gith < Formula
  desc "Simple Git repository manager TUI: see all your repos at a glance and run the everyday actions"
  homepage "https://github.com/MarlonRX/git-hero"
  url "https://github.com/MarlonRX/git-hero/archive/refs/tags/v0.4.0.tar.gz"
  sha256 "461092fddba47a46c02d7f7f55bb6f5c14f43101201099ccba6de195c2346848"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
  end

  test do
    assert_match "gith", shell_output("#{bin}/gith --version 2>&1 || true")
  end
end
