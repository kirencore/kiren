class Kiren < Formula
  desc "High-performance JavaScript runtime built with Rust"
  homepage "https://github.com/mertcanaltin/kiren"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/mertcanaltin/kiren/releases/download/v0.1.0/kiren-macos-arm64.tar.gz"
    sha256 "SHA256_ARM64_HERE"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/mertcanaltin/kiren/releases/download/v0.1.0/kiren-macos-x64.tar.gz"
    sha256 "SHA256_X64_HERE"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/mertcanaltin/kiren/releases/download/v0.1.0/kiren-linux-x64.tar.gz"
    sha256 "SHA256_LINUX_HERE"
  end

  def install
    bin.install "kiren"

    # Generate completions
    generate_completions_from_executable(bin/"kiren", "--completions")
  end

  test do
    # Test basic functionality
    (testpath/"hello.js").write("console.log('Hello from Kiren!');")
    assert_match "Hello from Kiren!", shell_output("#{bin}/kiren hello.js")

    # Test REPL help
    assert_match "Kiren", shell_output("#{bin}/kiren --help")

    # Test HTTP server creation
    (testpath/"server.js").write(<<~EOS
      const server = http.createServer();
      console.log("Server created:", typeof server);
    EOS
    )
    assert_match "Server created: object", shell_output("#{bin}/kiren server.js")
  end
end