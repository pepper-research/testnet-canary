EXIT_VAL=0

# format for future crate additions
# cd crates/<crate_path>
# cargo test -- --nocapture
# if [ $? -ne 0 ]
# then
#     exitval=1
# fi
# cd <back to root>

# LUT
cd crates/oracle/lut
# cargo test -- --nocapture
RUST_MIN_STACK=8388608 RUST_BACKTRACE=1 cargo test -- --nocapture # temporarily increase stack size to 8mb (we need about 2.5mb for due to some stack overflow issues)
if [ $? -ne 0 ] 
then
    EXIT_VAL=1
fi
cd ../../..

# TIME
cd crates/time
cargo test -- --nocapture
if [ $? -ne 0 ] 
then
    EXIT_VAL=1
fi
cd ../..

exit $EXIT_VAL