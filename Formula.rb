class RankedSearcher < Formula
  desc "Search inside text files using tf-idf formula, showing the most relevant search at the top"
  homepage "https://github.com/larscom/ranked-searcher"
  version "0.0.2"

  on_macos do
    on_intel do
      url "https://github.com/larscom/ranked-searcher/releases/download/0.0.2/ranked-searcher-0.0.2-macos-x86_64.tar.gz"
      sha256 "2b3cdcbac23a84457a308d833fc7e8f32b5b1695409c5513c7079c472eb9fbf3"
    end

    on_arm do
      url "https://github.com/larscom/ranked-searcher/releases/download/0.0.2/ranked-searcher-0.0.2-macos-arm64.tar.gz"
      sha256 "ebb2fd2ef231a94e862e9a5d534656b389c00aed01dd2e04e797587307e3c254"
    end
  end

  def install
    bin.install "ranked-searcher"
  end

  test do
    system "#{bin}/ranked-searcher", "test"
  end
end
