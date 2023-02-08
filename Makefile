.DEFAULT_GOAL 	:= help
.PHONY: compile run run-cli test coverage clients logs all gomod_tidy go_fmt help


# Commands
echo: # echo
	@echo hello

buildaws: # runs build and push to dockerhub
	@docker build --platform linux/amd64 -t 0xkeivin/pubrepo:medi0-core-server . 
	@docker push 0xkeivin/pubrepo:medi0-core-server

buildlocal: # runs build and push to dockerhub
	@docker build -t medi0-core-server . --no-cache

runlocal: # runs docker locally
	@docker run -p 8080:8080 medi0-core-server

runaws: # runs docker aws
	@docker run -p 8080:8080 0xkeivin/pubrepo:medi0-core-server &