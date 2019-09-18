pragma solidity ^0.5.9;

import 'openzeppelin-solidity/contracts/token/ERC20/SafeERC20.sol';

//Beneficieries (validators) template
import "../helpers/BeneficiaryOperations.sol";

contract DAIBridge is  BeneficiaryOperations {

        address private _pendingBeneficiary;

    /**
        * @notice Constructor.
        * @param _token  Address of DAI token
        */

        constructor (IERC20 _token) public
            BeneficiaryOperations(_token, msg.sender, _releaseTime) {
        }  

        // MODIFIERS
        /**
        * @dev Allows to perform method by existing beneficiary
        */
        modifier onlyExistingBeneficiary(address _beneficiary) {
            require(isExistBeneficiary(_beneficiary), "address is not in beneficiary array");
             _;
        }

        function sendAmount(uint value, string memory substrateAddress) public {

        }



}