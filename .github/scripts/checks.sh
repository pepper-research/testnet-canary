EXIT_VAL=0

# format for future crate additions
# cd crates/<crate_path>
# cargo check
# if [ $? -ne 0 ]
# then
#     exitval=1
# fi
# cd <back to root>

# AAOB
cd crates/aaob-module
cargo check
if [ $? -ne 0 ] 
then
    EXIT_VAL=1
fi
cd ../..

exit $EXIT_VAL