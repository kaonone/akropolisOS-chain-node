pragma solidity ^0.5.0;

import "../../contracts/helpers/BeneficiaryOperations.sol";


contract BeneficiaryOperationsImpl is BeneficiaryOperations {

    uint public value;

    function setValue(uint _value) public onlyManyBeneficiaries {
        value = _value;
    }

    function setValueAny(uint _value) public onlyAnyBeneficiary {
        value = _value;
    }

    function setValueAll(uint _value) public onlyAllBeneficiaries {
        value = _value;
    }

    function setValueSome(uint _value, uint howMany) public onlySomeBeneficiaries(howMany) {
        value = _value;
    }

    function nestedFirst(uint _value) public onlyManyBeneficiaries {
        nestedSecond(_value);
    }

    function nestedSecond(uint _value) public onlyManyBeneficiaries {
        value = _value;
    }

    //

    function nestedFirstAllToAll(uint _value) public onlyAllBeneficiaries {
        nestedSecondAllToAll(_value);
    }

    function nestedFirstAllToAll2(uint _value) public onlyAllBeneficiaries {
        this.nestedSecondAllToAll(_value); // this.
    }

    function nestedSecondAllToAll(uint _value) public onlyAllBeneficiaries {
        value = _value;
    }

    //

    function nestedFirstAnyToAny(uint _value) public onlyAnyBeneficiary {
        nestedSecondAnyToAny(_value);
    }

    function nestedFirstAnyToAny2(uint _value) public onlyAnyBeneficiary {
        this.nestedSecondAnyToAny(_value); // this.
    }

    function nestedSecondAnyToAny(uint _value) public onlyAnyBeneficiary {
        value = _value;
    }

    //

    function nestedFirstManyToSome(uint _value, uint howMany) public onlyManyBeneficiaries {
        nestedSecondSome(_value, howMany);
    }

    function nestedFirstAnyToSome(uint _value, uint howMany) public onlyAnyBeneficiary {
        nestedSecondSome(_value, howMany);
    }

    function nestedSecondSome(uint _value, uint howMany) public onlySomeBeneficiaries(howMany) {
        value = _value;
    }

}