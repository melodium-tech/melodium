#!/usr/bin/env bash

set -e
FAILURES=0
declare -i FAILURES

rm -f output*

text="
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam convallis dolor eget eros vestibulum, tristique porta nunc lacinia.
Præsent sed rutrum felis. Sed id sollicitudin neque. Cras volutpat urna arcu. Mauris felis tellus, mollis ac commodo sit amet,
laoreet at nibh. Nunc ac enim vel nibh rutrum posuere vitæ ut enim. Duis justo ex, dapibus sit amet nisi et, rhoncus vehicula orci.
Præsent suscipit risus ligula, ac dictum lacus mattis at."

echo "$text" | "$MELODIUM" pipe.mel > "output.log"

test "$(cat output.txt)" == "$text"
test "$(cat output.log)" == "$text"

exit $FAILURES


