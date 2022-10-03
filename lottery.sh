export CONTRACT=dev-1664736275925-68636536627316
export OWNER=rmlsnk.testnet
export ONE_NEAR=1000000000000000000000000
export THREE_NEAR=3000000000000000000000000
export FIVE_NEAR=5000000000000000000000000
export TEN_NEAR=10000000000000000000000000

export USER_1=participant_1.testnet
export USER_2=participant_2.testnet
export USER_3=participant_3.testnet
export USER_4=participant_4.testnet
export USER_5=participant_5.testnet
export USER_6=participant_6.testnet
export USER_7=participant_7.testnet
export USER_8=participant_8.testnet
export USER_9=participant_9.testnet
export USER_10=participant_10.testnet
export USER_11=participant_11.testnet
export USER_12=participant_12.testnet
export USER_13=participant_13.testnet
export USER_14=participant_14.testnet
export USER_15=participant_15.testnet
export USER_16=participant_16.testnet
export USER_17=participant_17.testnet
export USER_18=participant_18.testnet
export USER_19=participant_19.testnet
export USER_20=participant_20.testnet
export USER_21=participant_21.testnet
export USER_22=participant_22.testnet
export USER_23=participant_23.testnet
export USER_24=participant_24.testnet
export USER_25=participant_25.testnet
export USER_26=participant_26.testnet
export USER_27=participant_27.testnet
export USER_28=participant_28.testnet
export USER_29=participant_29.testnet
export USER_30=participant_30.testnet
export USER_31=participant_31.testnet
export USER_32=participant_32.testnet
export USER_33=participant_33.testnet
export USER_34=participant_34.testnet
export USER_35=participant_35.testnet
export USER_36=fungible_multisender.testnet
export USER_37=dev-1664736275925-68636536627316
export USER_38=dev-1664735236805-57354604957630
export USER_39=dev-1662975414616-70327018481235
export USER_40=participant_40.testnet
export USER_41=participant_41.testnet
export USER_42=participant_42.testnet
export USER_43=participant_43.testnet
export USER_44=participant_44.testnet
export USER_45=participant_45.testnet
export USER_46=duckhunter_test.testnet
export USER_47=freewind.testnet
export USER_48=dev-1664736275925-68636536627432
export USER_49=dev-1664735236805-57354604957430
export USER_50=dev-1662975414616-70327018481535
export GAS=300000000000000

END=50
EXPECTED_LOTTERY_ID=3
for ((i=0;i<=END;i++)); do
    user=USER_$i
    echo ${!user}
    near call $CONTRACT draw_near_enter '{
        "lottery_type": "BIG_LOTTERY",
        "num_participants": 50,
        "entry_fee": "'$ONE_NEAR'"
    }' --accountId=${!user} --depositYocto=$ONE_NEAR --gas=$GAS
    near view $CONTRACT get_lottery '{
        "lottery_id": EXPECTED_LOTTERY_ID
    }'
done

END=5
EXPECTED_LOTTERY_ID=4
for ((i=0;i<=END;i++)); do
    user=USER_$i
    echo ${!user}
    near call $CONTRACT draw_near_enter '{
        "lottery_type": "SIMPLE_LOTTERY",
        "num_participants": 5,
        "entry_fee": "'$ONE_NEAR'"
    }' --accountId=${!user} --depositYocto=$ONE_NEAR --gas=$GAS
    near view $CONTRACT get_lottery '{
        "lottery_id": EXPECTED_LOTTERY_ID
    }'
done

