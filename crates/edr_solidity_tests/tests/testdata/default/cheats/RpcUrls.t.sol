// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.18;

import "ds-test/test.sol";
import "cheats/Vm.sol";

contract RpcUrlTest is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // returns the correct url
    function testCanGetRpcUrl() public {
        string memory url = vm.rpcUrl("rpcAlias"); // note: this alias is pre-configured in the test runner
        string memory expected = vm.rpcUrl("rpcAliasMainnet");
        assertEq(url, expected);
    }

    // returns an error if env alias does not exist
    function testRevertsOnMissingEnv() public {
        vm._expectCheatcodeRevert("invalid rpc url: rpcUrlEnv");
        string memory url = vm.rpcUrl("rpcUrlEnv");
    }
}