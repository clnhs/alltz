class Alltz < Formula
  desc "🌍 Terminal-based timezone viewer for developers and remote teams"
  homepage "https://github.com/your-username/alltz"
  version "0.1.0"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/your-username/alltz/releases/download/v0.1.0/alltz-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_ARM64_HASH"
    else
      url "https://github.com/your-username/alltz/releases/download/v0.1.0/alltz-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64_HASH"
    end
  end

  on_linux do
    url "https://github.com/your-username/alltz/releases/download/v0.1.0/alltz-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "SHA256_LINUX_HASH"
  end

  def install
    bin.install "alltz-#{Hardware::CPU.arch}-apple-darwin" => "alltz" if OS.mac?
    bin.install "alltz-x86_64-unknown-linux-gnu" => "alltz" if OS.linux?
  end

  test do
    assert_match "alltz 0.1.0", shell_output("#{bin}/alltz --version")
    
    # Test CLI commands
    assert_match "Available Timezones", shell_output("#{bin}/alltz list")
    assert_match "Current time", shell_output("#{bin}/alltz time London")
  end
end