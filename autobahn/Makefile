AUTOBAHN_TESTSUITE_DOCKER := crossbario/autobahn-testsuite:0.8.2@sha256:5d4ba3aa7d6ab2fdbf6606f3f4ecbe4b66f205ce1cbc176d6cdf650157e52242

build-server:
	cargo build --release --example echo_server

run-server: build-server
	echo ${PWD}
	docker run -d  \
		--name fuzzingserver \
		-u `id -u`:`id -g` \
		-v ${PWD}/fuzzingclient.json:/fuzzingclient.json:ro \
		-v ${PWD}/reports:/reports \
		-p 9001:9001 \
		--net=host \
		--rm \
		$(AUTOBAHN_TESTSUITE_DOCKER) \
	  wstest -m fuzzingclient -s fuzzingclient.json
	../target/release/examples/echo_server
	