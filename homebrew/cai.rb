# Homebrew formula for CAI (Coding Agent Insights)
# To install:
#   brew install duyet/tap/cai
#   OR
#   brew tap duyet/tap && brew install cai

class Cai < Formula
  desc "SQL-like query tool for analyzing AI coding history"
  homepage "https://github.com/duyet/coding-agent-insights"
  url "https://github.com/duyet/coding-agent-insights/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "3f194c7dda84ef323676564e2cfbfb697e38280cad37bdcf500a0a16bdf14c2a"
  license "MIT OR Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--workspace", "--bins", "--locked", "--root", prefix, "--path", "."
  end

  test do
    # Test basic functionality
    system bin/"cai", "--version"
    system bin/"cai", "help"
  end
end
