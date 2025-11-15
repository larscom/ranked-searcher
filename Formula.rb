class RankedSearcher < Formula
  desc "Search inside text files using tf-idf formula, showing the most relevant search at the top"
  homepage "https://github.com/larscom/ranked-searcher"
  version "0.1.0"

  on_macos do
    on_intel do
      url "https://github.com/larscom/ranked-searcher/releases/download/0.1.0/ranked-searcher-0.1.0-macos-x86_64.tar.gz"
      sha256 "3c237b7c98ddc2e85925731cdd01c9915b65cdd4383c63118f9a8a1a86580e3e"
    end

    on_arm do
      url "https://github.com/larscom/ranked-searcher/releases/download/0.1.0/ranked-searcher-0.1.0-macos-arm64.tar.gz"
      sha256 "75b2031d5ee8242168bd5126c1194abd8fc9181f864370cd6d9043ad8a18ef6d"
    end
  end

  def install
    bin.install "ranked-searcher"
  end

  test do
    system "#{bin}/ranked-searcher", "test"
  end
end
