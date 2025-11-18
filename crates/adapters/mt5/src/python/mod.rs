// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

 //! Configuration structures for the MetaTrader 5 adapter.
 //!
 //! This module defines the configuration structures for the MT5 adapter.
 //! Mt5Config describes the HTTP endpoints and timeouts of the MT5 bridge.
 //! The credentials (login/password/server) are carried by `Mt5Credential` (common/credential.rs).
// This module intentionally left minimal - all PyO3 bindings are in bindings.rs