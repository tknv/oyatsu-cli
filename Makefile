install:
	install -Dm755 target/release/oyatsu /usr/local/bin/oyatsu
	install -Dm644 oyatsu.1 /usr/local/share/man/man1/oyatsu.1
