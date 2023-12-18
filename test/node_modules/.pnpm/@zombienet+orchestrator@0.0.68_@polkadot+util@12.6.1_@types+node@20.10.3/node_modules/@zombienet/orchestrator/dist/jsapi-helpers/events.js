"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.findPatternInSystemEventSubscription = void 0;
const debug = require("debug")("zombie::js-helpers::events");
function findPatternInSystemEventSubscription(api, re, timeout) {
    return __awaiter(this, void 0, void 0, function* () {
        let found = false;
        found = yield new Promise((resolve) => {
            const limitTimeout = setTimeout(() => {
                debug(`Timeout getting pattern (${timeout})`);
                resolve(false);
            }, timeout * 1000);
            api.query.system.events((events) => {
                let eventString = "";
                const matchedEvent = events.find((record) => {
                    eventString = "";
                    // extract the phase, event and the event types
                    const { event, phase } = record;
                    const types = event.typeDef;
                    eventString += `${event.section} : ${event.method} :: phase=${phase.toString()}\n`;
                    eventString += event.meta.docs.toString();
                    // loop through each of the parameters, displaying the type and data
                    event.data.forEach((data, index) => {
                        eventString += `${types[index].type};${data.toString()}`;
                    });
                    debug(eventString);
                    return re.test(eventString);
                });
                if (matchedEvent) {
                    debug(eventString);
                    clearTimeout(limitTimeout);
                    return resolve(true);
                }
            });
        });
        return found;
    });
}
exports.findPatternInSystemEventSubscription = findPatternInSystemEventSubscription;
