install:
	install -Dm755 target/release/oyatsu /usr/bin/oyatsu
	install -Dm644 oyatsu.1 /usr/share/man/man1/oyatsu.1
