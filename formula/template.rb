class RankedSearcher < Formula
  desc "Search inside text files using tf-idf formula, showing the most relevant search at the top"
  homepage "https://github.com/larscom/ranked-searcher"
  version "{{version}}"

  on_macos do
    on_intel do
      url "https://github.com/larscom/ranked-searcher/releases/download/{{version}}/ranked-searcher-{{version}}-macos-x86_64.tar.gz"
      sha256 "{{sha256_macos_intel}}"
    end

    on_arm do
      url "https://github.com/larscom/ranked-searcher/releases/download/{{version}}/ranked-searcher-{{version}}-macos-arm64.tar.gz"
      sha256 "{{sha256_macos_arm}}"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/larscom/ranked-searcher/releases/download/{{version}}/ranked-searcher-{{version}}-linux-x86_64.tar.gz"
      sha256 "{{sha256_linux_intel}}"
    end
  end

  def install
    bin.install "ranked-searcher"
  end

  test do
    system "#{bin}/ranked-searcher", "test"
  end
end
