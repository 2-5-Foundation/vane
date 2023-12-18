"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const mocha_1 = __importDefault(require("mocha"));
const utils_1 = require("@zombienet/utils");
const { EVENT_RUN_END, EVENT_TEST_FAIL, EVENT_TEST_PASS, EVENT_TEST_BEGIN } = mocha_1.default.Runner.constants;
let bannerPrinted = false;
class TestReporter {
    constructor(runner) {
        const stats = runner.stats;
        const logTableInit = new utils_1.CreateLogTable({
            head: [
                {
                    colSpan: 2,
                    hAlign: "center",
                    content: utils_1.decorators.green("Test Results"),
                },
            ],
            colWidths: [30, 100],
        });
        const logTable = new utils_1.CreateLogTable({
            colWidths: [30, 100],
        });
        const announcement = new utils_1.CreateLogTable({
            colWidths: [120],
        });
        announcement.pushToPrint([
            [
                utils_1.decorators.green("🛎️ Tests are currently running. Results will appear at the end"),
            ],
        ]);
        runner
            .once(EVENT_TEST_BEGIN, () => {
            announcement.print();
        })
            .on(EVENT_TEST_BEGIN, () => {
            if (!bannerPrinted)
                logTableInit.print();
            bannerPrinted = true;
        })
            .on(EVENT_TEST_PASS, (test) => {
            new utils_1.CreateLogTable({
                colWidths: [30, 100],
            }).pushToPrint([
                [
                    new Date().toLocaleString(),
                    `✅ ${test.title} ${utils_1.decorators.underscore(`(${test.duration}ms)`)}`,
                ],
            ]);
        })
            .on(EVENT_TEST_FAIL, (test) => {
            new utils_1.CreateLogTable({
                colWidths: [30, 100],
            }).pushToPrint([
                [
                    new Date().toLocaleString(),
                    `❌ ${test.title} ${utils_1.decorators.underscore(`(${test.duration}ms)`)}`,
                ],
            ]);
        })
            .once(EVENT_RUN_END, () => {
            logTable.pushTo([
                [
                    {
                        colSpan: 2,
                        hAlign: "left",
                        content: `Result: ${stats.passes}/${stats.passes + stats.failures}`,
                    },
                ],
            ]);
            logTable.print();
        });
    }
}
module.exports = TestReporter;
