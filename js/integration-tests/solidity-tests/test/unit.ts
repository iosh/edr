import { assert } from "chai";
import { TestContext } from "./testContext";

describe("Unit tests", () => {
  let testContext: TestContext;

  before(async () => {
    testContext = await TestContext.setup();
  });

  it("SuccessAndFailure", async function () {
    const { totalTests, failedTests } = await testContext.runTestsWithStats(
      "SuccessAndFailureTest"
    );

    assert.equal(failedTests, 1);
    assert.equal(totalTests, 2);
  });

  it("ContractEnvironment", async function () {
    const { totalTests, failedTests } = await testContext.runTestsWithStats(
      "ContractEnvironmentTest",
      {
        sender: Buffer.from("976EA74026E726554dB657fA54763abd0C3a0aa9", "hex"),
        chainId: 12n,
        blockNumber: 23n,
        blockTimestamp: 45n,
      }
    );

    assert.equal(failedTests, 0);
    assert.equal(totalTests, 1);
  });

  describe("IsolateMode", function () {
    it("IsolateMode on", async function () {
      const { totalTests, failedTests } = await testContext.runTestsWithStats(
        "IsolateTest",
        {
          isolate: true,
        }
      );

      assert.equal(failedTests, 0);
      assert.equal(totalTests, 1);
    });

    it("IsolateMode off", async function () {
      const { totalTests, failedTests } =
        await testContext.runTestsWithStats("IsolateTest");

      assert.equal(failedTests, 1);
      assert.equal(totalTests, 1);
    });
  });

  describe("TestFail", function () {
    it("TestFail on", async function () {
      const { totalTests, failedTests } = await testContext.runTestsWithStats(
        "TestFailTest",
        {
          testFail: true,
        }
      );

      // Reverting test starting with `testFail` should be reported as success if `testFail` is on
      assert.equal(failedTests, 0);
      assert.equal(totalTests, 1);
    });

    it("TestFail off", async function () {
      const { totalTests, failedTests } =
        await testContext.runTestsWithStats("TestFailTest");

      // Reverting test starting with `testFail` should be reported as failure if `testFail` is off
      assert.equal(failedTests, 1);
      assert.equal(totalTests, 1);
    });
  });

  it("GlobalFork", async function () {
    if (testContext.rpcUrl === undefined) {
      this.skip();
    }

    const { totalTests, failedTests } = await testContext.runTestsWithStats(
      "GlobalForkTest",
      {
        ethRpcUrl: testContext.rpcUrl,
        forkBlockNumber: 20_000_000n,
      }
    );

    assert.equal(failedTests, 0);
    assert.equal(totalTests, 1);
  });

  it("ForkCheatcode", async function () {
    if (testContext.rpcUrl === undefined) {
      this.skip();
    }

    const { totalTests, failedTests } = await testContext.runTestsWithStats(
      "ForkCheatcodeTest",
      {
        rpcEndpoints: {
          alchemyMainnet: testContext.rpcUrl,
        },
      }
    );

    assert.equal(failedTests, 0);
    assert.equal(totalTests, 1);
  });
});