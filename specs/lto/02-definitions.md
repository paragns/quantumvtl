# 2. Definitions, symbols, abbreviations, and conventions

## 2.1 Definitions

This clause defines the special terms, abbreviations, and acronyms that are used in this publication. If the term
being looked for is not found, refer to http://www-01.ibm.com/software/globalization/terminology/index.jsp.

- **2.1.1 2:1 compression**: The relationship between the quantity of data that can be stored with compression as compared to the quantity of data that can be stored without compression. For example, with 2:1 compression, twice as much data can be stored with compression as can be stored without compression.
- **2.1.2 abend**: Abnormal end of task.
- **2.1.3 access method**: A technique for moving data between processor storage and input/output devices.
- **2.1.4 adapter**: adapter card.
- **2.1.5 adapter card**: A circuit card that adds function to a computer.
- **2.1.6 ADC**: Automation/Drive Interace Commands (ADC).
- **2.1.7 ADI**: Automation/Drive Interface (ADI).
- **2.1.8 ADT**: Automation/Drive Interface Transport Protocol (ADT).
- **2.1.9 AES**: Advanced Encryption Standard.
- **2.1.10 AK**: Authentication Key
- **2.1.11 AL_PA**: Arbitrated Loop Physical Address (ALPA; AL_PA).
- **2.1.12 ALPA**: Arbitrated Loop Physical Address (ALPA; AL_PA).
- **2.1.13 ANSI**: American National Standards Institute.
- **2.1.14 Arbitrated Loop**: A Fibre Channel Loop topology protocol, also known as L-port.
- **2.1.15 Arbitrated Loop Physical Address (ALPA; AL_PA)**: An 8-bit value that identifies a device in an arbitrated loop.
- **2.1.16 archiving**: The storage of backup files and associated journals, usually for a given period of time.
- **2.1.17 archiving application**: The retention of records, in machine-readable form, for historical purposes.
- **2.1.18 argument**: Any value of an independent variable.
- **2.1.19 ASC**: Additional Sense Code.
- **2.1.20 ASCII**: American Standard Code for Information Interchange. When used to describe a field, indicates that the field contains only ASCII printable characters (i.e., code values 20h to 7Eh) and may be terminated with one or more ASCII null (00h) characters.
- **2.1.21 ASCQ**: Additional Sense Code Qualifier.
- **2.1.22 ASN.1**: Abstract Syntax Notation One - OSI's encoding (see X.208 standard)
- **2.1.23 Automation/Drive Interace Commands (ADC)**: A T10 standard that describes the commands that are used for communication between an Automation device (i.e., Library) and a Drive (i.e., tape drive). This command set standard is on the third generation at the time of this publication. See Project T10/1895-D, Information technology - Automation/Drive Interface Commands - 3 (ADC-3).
- **2.1.24 Automation/Drive Interface (ADI)**: The umbrella under which the T10 standards address the interface between removable media library controllers and the physical drives resident in those libraries. The standards defined are the Automation/Drive Interace Commands (ADC) and the Automation/Drive Interface Transport Protocol (ADT).
- **2.1.25 Automation/Drive Interface Transport Protocol (ADT)**: The standard covering the Automation Drive Interface - Transport Protocol. This specific document covers the transport mechanisms between removable media library controllers and the physical drives resident in those libraries, specifically the encapsulation, logical transmission, and end-point delivery and reception of the commands associated with the ADI effort. At the time this document was published this standard was in it's second revision. T10/1742-D, Information technology - Automation/Drive Interface - Transport Protocol -2 (ADT-2)
- **2.1.26 backups**: The short-term retention of records used for restoring essential business and server files when vital data has been lost.
- **2.1.27 beginning of tape (BOT)**: The location on a magnetic tape that indicates the beginning of the permissible recording area.
- **2.1.28 BER**: Basic Encoding Rules - used with ASN.1 (see X.209 standard)
- **2.1.29 bezel**: The frame that fits over the front of the tape drive. This includes a button and a message display.
- **2.1.30 bit**: The smallest unit of data in a computer. A bit (short for binary digit) has a single binary value or either 0b or 1b.
- **2.1.31 BOP**: Beginning of Partition - logical beginning of a data area (logical object 0)
- **2.1.32 BOT**: Beginning of tape.
- **2.1.33 bpi**: Bits per inch.
- **2.1.34 BPI**: Bytes per inch.
- **2.1.35 buffer**: A routine or storage used to compensate for a difference in rate of flow of data, or time of occurrence of events, when transferring data from one device to another.
- **2.1.36 buffered mode**: The buffered mode allows a number of logical objects to accumulate in the object buffer before the data is transferred to the medium or host device.
- **2.1.37 byte**: A byte is a unit of data comprised of 8 bits.
- **2.1.38 CA**: Contingent allegiance.
- **2.1.39 capacity**: See media capacity.
- **2.1.40 cartridge**: See tape cartridge.
- **2.1.41 cartridge memory (CM)**: A non-contact electronic module embedded in the cartridge that can be used to store and retrieve information.
- **2.1.42 CC**: Check Condition.
- **2.1.43 CDB**: Command descriptor block.
- **2.1.44 cleaning cartridge**: A tape cartridge that is used to clean the heads of a tape drive. Contrast with data cartridge.
- **2.1.45 command**: A control signal that initiates an operation or the beginning of a sequence of operations.
- **2.1.46 command timeout**: A host controlled period of time, following the issuance of a command where the host has not received a status response for that command.
- **2.1.47 compaction**: See data compression.
- **2.1.48 compression**: See data compression.
- **2.1.49 contingent allegiance**: (1) A condition in which a drive owes a response to a specific channel path because of a unit check. (2) A condition generated by a check condition status during which a target preserves sense data.
- **2.1.50 conversion**: The process of changing from one method of data processing to another or from one data-processing system to another.
- **2.1.51 data**: Any representations such as characters or analog quantities to which meaning is, or might be, assigned.
- **2.1.52 data cartridge**: A tape cartridge that is dedicated to storing data. Contrast with cleaning cartridge.
- **2.1.53 data compression**: An algorithmic data-reduction technique that encodes data from the host and stores it in less space than unencoded data. The original data is recovered by an inverse process called decompression.
- **2.1.54 data compression ratio**: The number of host data bytes divided by the number of encoded bytes. It is variable depending on the characteristics of the data being processed. The more random the data stream, the lower the opportunity to achieve compression.
- **2.1.55 data transfer rate**: The amount of data that can be stored on a tape cartridge with respect to time.
- **2.1.56 dataset**: The major unit of data storage and retrieval, consisting of a collection of data in one of several prescribed arrangements and described by control information to which the system has access.
- **2.1.57 DCC**: Deferred Check Condition, also known as deferred unit check.
- **2.1.58 deferred unit check**: A condition in which a drive returns a unit check indication for an event that occurred asynchronously with the channel commands. The deferred unit check normally does not refer to the command that receives the indication.
- **2.1.59 DER**: Distinguished Encoding Rules - a subset of BER
- **2.1.60 application design capacity**: The lower range of media capacity that is expected and that should be accommodated. See "Application design capacity {LP17h:0018h}" on page 304..
- **2.1.61 device**: Any hardware component or peripheral that can receive and transmit data, such as a tape drive or tape library.
- **2.1.62 device driver**: An executable file or program installed on a host system used to control or access a device.
- **2.1.63 diagnostic**: A test or procedure designed to detect, recognize, locate, isolate or explain faults in equipment or errors in programs.
- **2.1.64 diagnostic cartridge**: A tape cartridge used to perform a diagnostic.
- **2.1.65 digest**: a cryptographically strong hash (i.e., SHA-x, MD-x)
- **2.1.66 DK**: Data Key - key used for encryption/decryption
- **2.1.67 DKi**: Data Key Identifier - a field in the EEDK(s)/SEDK and part of the DKi/IV recorded on media which associates the encryption of the record to EEDK(s) and ultimately a DK
- **2.1.68 DKi/IV**: Combined DKi and IV prepended to each record in the logical format
- **2.1.69 drive**: A device used to store data to media and subsequently restore data from media.
- **2.1.70 drive dump**: The recording, at a particular instant, of the contents of debug information into a buffer or onto medium for the purpose of retrieval for debug purposes.
- **2.1.71 drive head**: The component of a tape drive which converts and records an electrical signal to a magnetic signal on tape, and subsequently detects and converts such signals.
- **2.1.72 drive loaded**: A condition of a tape drive in which a tape cartridge has been inserted in the drive, and the tape has been threaded to the beginning-of-partition 0 position.
- **2.1.73 effective data rate**: The average number of a unit of data per unit time transferred from a data source to a data sink and accepted as valid. For example, the rates may be expressed in bits per second (bps), bytes per second (Bps), megabytes per second (MB/s), terabytes per hour (TB/hr), etc..
- **2.1.74 eject**: To remove or force from within. Generally refers to the last part of the unload process to allow removal of a tape cartridge from the drive.
- **2.1.75 EKM**: External Key Manager
- **2.1.76 enable**: To provide the means or opportunity. The modification of system, control unit, or device action through the change of a software module or a hardware switch (circuit jumper) position.
- **2.1.77 enclosure**: A device, such as a desktop unit, tape cartridge autoloader, or tape library, into which a tape drive may be installed.
- **2.1.78 EOD**: End Of Data - a dataset denoting the end of user data
- **2.1.79 EOP**: End of partition. This usually refers to Logical End of Partition (LEOP), but may refer to Physical End of Partition (PEOP).
- **2.1.80 EOT**: End of tape. This may refer to the physical end of tape or the logical end of tape.
- **2.1.81 ERA**: Error-recovery action.
- **2.1.82 ERP**: See error-recovery procedures (ERP)
- **2.1.83 error log**: Maintained by the drive, a list that contains recent error codes. The codes identify errors that pertain to the drive.
- **2.1.84 error-recovery procedures (ERP)**: (1) Procedures designed to help isolate and, where possible, to recover from errors in equipment. The procedures are often used in conjunction with programs that record the statistics of machine malfunctions. (2) Error-recovery procedures performed by the subsystem.
- **2.1.85 explicitly activated**: A process in which the attributes of an identifier are specified. Contrast with implicitly activated.
- **2.1.86 extended contingent allegiance**: (1) A condition caused by a permanent buffered-write error in which the drive responds only to the channel path group from which the write command was received. The extended contingent allegiance continues until a controlling computer in the channel path group retrieves the unwritten data from the buffer or issues a tape motion command. (2) A condition generated by an initiate recovery message to assist in extended error recovery procedures in multi-initiator systems.
- **2.1.87 F-port**: Fabric port.
- **2.1.88 FC**: Fibre Channel.
- **2.1.89 FCP**: Fibre Channel Protocol - the SCSI mapping to fibre channel
- **2.1.90 fiber**: A physical communication cable or connection used to attach two or more Fibre Channel devices.
- **2.1.91 Fibre Channel**: A standard interconnection interface used to attach host systems and/or peripheral devices.
- **2.1.92 FID**: Format Identification Dataset.
- **2.1.93 field replaceable unit (FRU)**: An assembly that is replaced in its entirety when any one of its components fails.
- **2.1.94 file**: A set of related records, treated as a unit; for example, in stock control, a file could consist of a set of invoices.
- **2.1.95 file protected**: Pertaining to a tape volume from which data can be read only. Data cannot be written on or erased from the tape.
- **2.1.96 filemark**: A logical object which is a demarcation, recorded on media, often used to separate files or provide other organizational structure to recorded data. Usage and convention of filemarks is controlled by the attached host system(s).
- **2.1.97 FIPS**: Federal Information Processing Standards
- **2.1.98 firmware**: Proprietary code that is usually delivered as part of an operating system or device. Firmware is more efficient than software loaded from an alterable medium, and is more adaptable to change than hardwired embedded logic.
- **2.1.99 FL-port**: Fabric loop port.
- **2.1.100 FMR**: Field microcode replacement.
- **2.1.101 format**: The arrangement or layout of data on a data medium.
- **2.1.102 FRU**: See field replaceable unit (FRU).
- **2.1.103 GB**: See gigabyte (GB).
- **2.1.104 Gb**: See gigabit (Gb).
- **2.1.105 GCM**: Galois Counter Mode
- **2.1.106 gigabit (Gb)**: 1 000 000 000 bits of storage.
- **2.1.107 gigabyte (GB)**: 1 000 000 000 bytes of storage.
- **2.1.108 hard addressing**: A method of specifying a fixed AL_PA address for a device in a Fibre Channel loop configuration.
- **2.1.109 hardware**: The physical equipment or components that form a device or system.
- **2.1.110 HBA**: host bus adapter.
- **2.1.111 head**: See drive head
- **2.1.112 host bus adapter**: A specific type of adapter card which provides the connection to a physical device interconnect such as Fibre Channel.
- **2.1.113 host system**: A data-processing system that is used to prepare programs and the operating environments for use on another computer or controller.
- **2.1.114 IBM Proprietary Protocol (IPP)**: IBM vendor-specific method of configuring and controlling encryption
- **2.1.115 IBM Tape Diagnostic Tool (ITDT)**: The ITDT Tool offers multiple functional capabilities that simplify the task of updating tape and library firmware. It is available for most major platforms and requires no special device drivers. See https://www.ibm.com/support/fixcentral/options?selectionBean.selectedTab=find&selection=System+Storage%3bibm%2fStorage_Tape%3bTape+drivers+and+software%3bibm%2fStorage_Tape%2fIBM+Tape+Diagnostic+Tool+ITDT
- **2.1.116 ID**: identifier
- **2.1.117 implicitly activated**: A process in which the attributes of an identifier are determined by default. Contrast with explicitly activated.
- **2.1.118 initiator**: A SCSI device that requests an I/O process to be performed by another SCSI device (a target). In some cases, an initiator can also be a target.
- **2.1.119 input/output (I/O)**: Data that is provided to a computer or data that results from computer processing.
- **2.1.120 install**: To set up for use or service. The act of adding a product, feature, or function to a system or device either by a singular change or by the addition of multiple components or devices.
- **2.1.121 interchange application**: The preparation of tapes for use on other systems or devices, either local or remote, or the use of tape data prepared by another system.
- **2.1.122 Internet**: The worldwide collection of interconnected networks that use the Internet suite of protocols and permit public access.
- **2.1.123 invoke**: To petition for help or support. The request for a feature or function to be utilized in future processing activities through the use of software or hardware commands.
- **2.1.124 I/O**: input/output (I/O).
- **2.1.125 IPP**: IBM Proprietary Protocol (IPP).
- **2.1.126 ITDT**: IBM Tape Diagnostic Tool (ITDT).
- **2.1.127 IV**: Initialization Vector - a value also called a nonce, used with a key for AES block ciphers
- **2.1.128 journaling**: Recording transactions against a dataset so that the dataset can be reconstructed by applying transactions in the journal against a previous version of the dataset.
- **2.1.129 KB**: See kilobyte.
- **2.1.130 kibibyte**: 1024 bytes of storage.
- **2.1.131 KiB**: See kibibyte.
- **2.1.132 kilobyte**: 1000 bytes of storage.
- **2.1.133 L-port**: Arbitrated Loop Fibre Channel host connection. May attach to a fabric (switch) FL-port.
- **2.1.134 LDI**: Library Drive Interface - a specific interface protocol for tape device to automation interface (over RS-422)
- **2.1.135 LEOT**: logical end of tape
- **2.1.136 Linear Tape-Open (LTO)**: A type of tape storage technology developed by the IBM Corporation, Hewlett-Packard, and Quantum (formerly Seagate). LTO technology is an "open format" technology, which means that its users have multiple sources of product and media. The "open" nature of LTO technology enables compatibility between different vendors' offerings by ensuring that vendors comply with verification standards.
- **2.1.137 LN_Port**: Fibre Channel host attachment configuration in which the drive attempts to negotiate first to Arbitrated Loop (NL-port), then Point-to-Point (N-port). May attach to a fabric (switch) F-port or FL-port. This may be thought of as L->N negotiation.
- **2.1.138 load**: Following the insertion of a tape cartridge into the device, the act of positioning the tape (performed by the drive) for subsequent reading or writing.
- **2.1.139 load point**: The beginning of the recording area on magnetic tape.
- **2.1.140 logical block**: A unit of data transfered between an initator and the drive. See record.
- **2.1.141 logical end of tape**: A point on the tape where written data normally ends.
- **2.1.142 logical object**: A logical block or a filemark.
- **2.1.143 LPOS**: Longitudinal Position.
- **2.1.144 LSB**: Least significant byte.
- **2.1.145 lsb**: Least significant bit.
- **2.1.146 LTO**: Linear Tape-Open (LTO).
- **2.1.147 LTO-DC**: LTO Data Compression (LTO-DC).
- **2.1.148 LTO Data Compression (LTO-DC)**: A method that compresses logical objects before the drive writes them to tape. LTO-DC encodes and detects record boundaries and file markers (which are encoded as control symbols). It also allows switching between compression and no compression within the data stream, which prevents data from expanding when the drive compresses random or encrypted data.
- **2.1.149 LUN**: Logical unit number.
- **2.1.150 MAC**: Message Authentication Code - a digest which validates encrypted data. Appended to each encrypted record in the logical format for cryptographic integrity validation
- **2.1.151 magnetic recording**: A technique of storing data by selectively magnetizing portions of a magnetizable material.
- **2.1.152 magnetic tape**: A tape with a magnetizable surface layer on which data can be stored by magnetic recording.
- **2.1.153 magnetic tape drive**: A mechanism for moving magnetic tape and controlling its movement.
- **2.1.154 MAM**: Medium Auxiliary Memory (MAM).
- **2.1.155 Management Information Base (MIB)**: A computing information repository used by Simple Network Management Protocol (SNMP)
- **2.1.156 manual mode**: A mode of operation that can be selected on a cartridge loader or library. This mode allows a single tape cartridge feed, performed by the operator.
- **2.1.157 MB**: See megabyte (MB).
- **2.1.158 Mb**: See megabit (Mb).
- **2.1.159 mebibit (Mib)**: 1 048 576 bits of storage (i.e., 2^20)
- **2.1.160 mebibyte (MiB)**: 1 048 576 bytes of storage (i.e., 2^20)
- **2.1.161 media**: Plural of medium.
- **2.1.162 media capacity**: The amount of data that can be contained on storage media and expressed in units of data, usually gigabyte (GB) or terabyte (TB).
- **2.1.163 medium**: A physical material in or on which information may be represented, such as magnetic tape.
- **2.1.164 Medium Auxiliary Memory (MAM)**: A non-volatile memory. MAM is used to store data that describes the media and its contents. MAM is usually stored on cartridge memory (CM).
- **2.1.165 megabit (Mb)**: 1 000 000 bits of storage (i.e., 10^6).
- **2.1.166 megabyte (MB)**: 1 000 000 bytes of storage (i.e., 10^6).
- **2.1.167 Mib**: mebibit (Mib).
- **2.1.168 MiB**: mebibyte (MiB).
- **2.1.169 microcode**: Embedded device programming which controls the behavior and functioning of the device.
- **2.1.170 microprocessor**: An integrated circuit that accepts coded instructions for execution; the instructions may be entered, integrated, or stored internally.
- **2.1.171 microsecond (us)**: One millionth of a second (0.000 001 s).
- **2.1.172 migration**: See conversion.
- **2.1.173 millisecond (ms)**: One thousandth of a second (0.001 s)
- **2.1.174 MIM**: Medium Information Message.
- **2.1.175 msb**: Most significant bit.
- **2.1.176 MSB**: Most significant byte.
- **2.1.177 N-port**: Point-to-Point Fibre Channel host connection. May attach to a fabric (switch) FL-port.
- **2.1.178 N/A**: Not Applicable.
- **2.1.179 native data transfer rate**: The amount of data that can be stored without compression on a tape cartridge with respect to time.
- **2.1.180 native storage capacity**: The amount of data that can be stored without compression on a tape cartridge.
- **2.1.181 NL_Port**: Fibre Channel host attachment configuration in which the drive port attempts to negotiate first to Point-to-Point (N-port), then Arbitrated Loop (NL-port). May attach to a fabric (switch) F-port or FL-port. This may be thought of as N->L negotiation.
- **2.1.182 node**: Fibre channel term for the logical connection to a device.
- **2.1.183 nominal capacity**: The nominal media capacity.
- **2.1.184 nonce**: number used once - a value used in conjunction with the key for AES block ciphers (also IV)
- **2.1.185 OEM**: Original equipment manufacturer.
- **2.1.186 offline**: An operating condition where the host system cannot interact with the drive through the specified interface.
- **2.1.187 online**: An operating condition where the host system can interact normally with the drive through the specified interface.
- **2.1.188 OOB**: Out-Of-Band
- **2.1.189 open system**: Computer systems whose operating standards and methods are not proprietary.
- **2.1.190 operating system**: The master computer control program that translates the user commands and allows software application programs to interact with the computer hardware and attached devices.
- **2.1.191 OSI**: Open Systems Interconnection - (see X.200 standard)
- **2.1.192 overwrite**: A write operation that records a logical object in a logical position that is not an append point (see 4.2.3).
- **2.1.193 parity**: The state of being even-numbered or odd-numbered. A parity bit is a binary number that is added to a group of binary numbers to make the sum of that group always odd (odd parity) or even (even parity) which is commonly used for error detection.
- **2.1.194 PEOT**: physical end of tape (PEOT)
- **2.1.195 physical end of tape (PEOT)**: A point on the tape beyond which the tape is not permitted to move.
- **2.1.196 PKCS**: Public-Key Cryptography Standards
- **2.1.197 POR**: Power-on reset.
- **2.1.198 port**: Fibre channel or SAS term for the physical connection to a device.
- **2.1.199 power-off**: To remove electrical power from a device.
- **2.1.200 power-on**: To apply electrical power to a device.
- **2.1.201 powered-on**: The state of a device when power has been applied to it.
- **2.1.202 primed**: Pertaining to a condition of a tape drive when the controlling computer addresses the drive but the drive is not in a ready state.
- **2.1.203 PRNG**: Pseudo Random Number Generator
- **2.1.204 processing application**: The execution of a systematic sequence of operations performed on data to accomplish a specific purpose.
- **2.1.205 protocol**: The meanings of, and the sequencing rules for, requests and responses that are used to manage a network, transfer data, and synchronize the states of network components.
- **2.1.206 quiesce**: To bring a device or system to a halt by a rejection of new requests for work.
- **2.1.207 read**: To acquire or interpret data from a storage device, from a data medium, or from another source.
- **2.1.208 read-type commands**: Any commands that cause data to be read from tape or affect buffered read data.
- **2.1.209 reboot**: To reinitialize the execution of a program by repeating the initial program load (IPL) operation.
- **2.1.210 record**: A logical object that contains user data (e.g., not a filemark).
- **2.1.211 recording density**: The number of bits in a single linear track measured per unit of length of the recording medium.
- **2.1.212 reset**: To return a device, circuit, or value to a clear state.
- **2.1.213 retension (or refresh)**: The process or function of tightening the tape onto the cartridge, if it is sensed that the tape has a loose wrap on the cartridge.
- **2.1.214 RS-422 connector**: Located at the rear of the device, the connector to which the internal RS-422 cable of an enclosure connects. The connection enables a library (i.e., medium changer) to communicate with the drive.
- **2.1.215 RS-422 interface**: An electrical interface standard that is approved by the Electronic Industries Association (EIA) for connecting serial devices.
- **2.1.216 RSA**: Method authored by Rivest, Shamir, Adleman
- **2.1.217 s**: second
- **2.1.218 SAN**: Storage Area Network.
- **2.1.219 SAS**: Serial Attached SCSI
- **2.1.220 SCSI**: Small Computer System Interface.
- **2.1.221 SCSI device**: A host adapter or a target controller that can be attached to the SCSI bus.
- **2.1.222 SCSI ID**: The identifier used to uniquely identify the address on the bus. When used on Fibre Channel devices this refers to the AL_PA.
- **2.1.223 SCSI Sense Data**: In response to a command from the server, a packet of SCSI sense bytes that contains information about the error that is sent back to the server by the drive in autosense. SCSI Sense Data may also be returned by the REQUEST SENSE command, but that sense data is usually unsolicited sense data and does not contain error information.
- **2.1.224 sense data**: SCSI Sense Data.
- **2.1.225 Serial Attached SCSI (SAS)**: A transport for exchanging information between SCSI devices using a standardized serial interconnect.
- **2.1.226 server**: A functional unit that provides services to one or more clients over a network. Synonymous with host system.
- **2.1.227 SHA**: Secure Hash Algorithm (can be SHA-1 (160 bit), or SHA-2 algorithms at differing bit strengths shortened to SHA-256, SHA-384, SHA-512, for bit size)
- **2.1.228 SIM**: Service Information Message.
- **2.1.229 Small Computer Systems Interface (SCSI)**: A standard used by computer manufacturers for attaching peripheral devices (such as tape drives, hard disks, CD-ROM players, printers, and scanners) to computers (servers). Pronounced "scuzzy."
- **2.1.230 soft addressing**: A method of specifying a standard arbitration method for assigning an AL_PA for a device in a Fibre Channel loop configuration.
- **2.1.231 software**: Programs, procedures, rules, and any associated documentation pertaining to the operation of a computer system.
- **2.1.232 special feature**: A specific design addition to an IBM product that is quoted in the IBM Sales Manual and ordered separately.
- **2.1.233 standard function**: The significant design elements of an IBM product that are included as part of the basic standard product.
- **2.1.234 Storage Area Network (SAN)**: A high-speed subnetwork of shared storage devices. A SAN's architecture makes all storage devices available to all servers on a LAN or WAN. As more storage devices are added to a SAN, they too will be accessible from any server in the larger network. Because stored data does not reside directly on any of a network's servers, server power is used for business applications, and network capacity is released to the end user.
- **2.1.235 switch**: A network infrastructure component to which multiple nodes attach. Unlike hubs, switches typically have the ability to switch node connections from one to another. A typical switch can facilitate several simultaneous bandwidth transmissions between different pairs of nodes.
- **2.1.236 synchronization**: The process of coordinating the activities of the controlling computer and the magnetic tape subsystem to obtain the condition in which the buffer is empty and the tape is in the correct position for the next operation.
- **2.1.237 T10**: ANSI group responsible for SCSI model and command sets, see http://www.t10.org
- **2.1.238 T11**: ANSI group responsible for FCP/fibre channel protocols, see http://www.t11.org
- **2.1.239 TB**: see terabyte (TB).
- **2.1.240 tape**: Commonly refers to magnetic tape or the tape cartridge.
- **2.1.241 tape cartridge**: A container holding magnetic tape that can be processed without separating it from the container.
- **2.1.242 tape drive**: A device that is used for moving magnetic tape and includes the mechanisms for writing and reading data to and from the tape.
- **2.1.243 tape unit**: A device that contains tape drives and their associated power supplies and electronics.
- **2.1.244 TapeAlert**: A patented technology and ANSI standard that defines conditions and problems that are experienced by tape drives.
- **2.1.245 TapeAlert flags**: Status and error messages that are generated by the TapeAlert utility and are reported to a host system.
- **2.1.246 target**: A SCSI device that performs an operation requested by the initiator.
- **2.1.247 target routine**: A target routine is an I/O process directed to a target, and not to a logical unit.
- **2.1.248 terabyte (TB)**: 1_000_000_000_000 bytes of storage.
- **2.1.249 topology**: In communications, the physical or logical arrangement of nodes in a network, especially the relationships among nodes and the links between them.
- **2.1.250 transfer rate**: data transfer rate.
- **2.1.251 TRNG**: True Random Number Generator
- **2.1.252 TSM**: Tivoli Storage Manager
- **2.1.253 tuple**: An ordered set of values or elements.
- **2.1.254 Type A cartridge**: A cartridge that has the capacity, density, and tracks as defined in the format specification for the generation that introduced this physical cartridge (e.g., for the Ultrium 7 cartridge, the U-732).
- **2.1.255 Type M cartridge**: A cartridge that provides a feature different than a Type A cartridge. For LTO-8, this is an Ultrium 7 cartridge that has a capacity of 9 000 GB and reports unique density information (see 5.2.26).
- **2.1.256 Type M eligible**: A cartridge that meets the definition of a new cartridge for the purpose of the cartridge being eligible to be changed to a Type M cartridge.
- **2.1.257 unload**: The act (performed by the drive) of unthreading tape from the drive's internal tape path and returning it (with the leader block) to the tape cartridge.
- **2.1.258 universal time (UT)**: The time at longitude zero, colloquially known as Greenwich Mean Time. See http://www.usno.navy.mil/USNO/time/master-clock/systems-of-time.
- **2.1.259 vital product data**: Non-volatile information including configuration, calibration, etc., used to control the behavior and operation of the device.
- **2.1.260 volume**: (1) A certain portion of data, together with its data carrier, that can be handled conveniently as a unit. (2) A data carrier that is mounted and demounted as a unit, for example, a reel of magnetic tape, a disk pack.
- **2.1.261 volume coherency set**: A set of information contained in logical objects including a volume coherency count (see 4.20) for which coherency across an entire volume is desired.
- **2.1.262 VPD**: Vital Product Data - information stored in drive nonvolatile memory
- **2.1.263 web**: World Wide Web (www).
- **2.1.264 World Wide Name**: A unique, 8-byte identifier that is assigned by IBM Manufacturing to each tape drive and used to identify a drive.
- **2.1.265 World Wide Web (www)**: A network of servers that contain programs and files. Many of the files contain hypertext links to other documents that are available through the network.
- **2.1.266 WORM (Write Once, Read Many)**: A write or append methodology for allowing data to be written only once, disallowing overwriting.
- **2.1.267 write**: To store or encode data to a storage device, to data medium, or to another source.
- **2.1.268 Write Once, Read Many (WORM)**: A write or append methodology for allowing data to be written only once, disallowing overwriting.
- **2.1.269 write protected**: A state disallowing write operations to a device or medium.
- **2.1.270 write-type commands**: Any commands that cause data to be written on tape or affect buffered write data.

## 2.2 Conventions

### 2.2.1 Radix representation

Binary numbers are represented by numbers followed by b. Hexadecimal numbers are represented by 0-9 and
A-F followed by h. Numbers with no suffix can be assumed to be decimal.

### 2.2.2 Bit Numbering

Bit numbering follows ANSI standards as follows:
- Bit 7 is the most significant bit (msb) occupying the leftmost bit position in the diagrams
- Bits 6 through 1 continue from left to right in descending order
- Bit 0 is the least significant bit (lsb) occupying the rightmost bit position in the diagrams

### 2.2.3 Units of measure for data storage

Decimal units such as KB, MB, GB, and TB have commonly been used to express data storage values. Some
environments, such as programming or memory values often use binary units such as KiB, MiB, GiB, and TiB. At
the kilobyte level, the difference between decimal and binary units of measurement is relatively small (2.4%).
This difference grows as data storage values increase, and when values reach terabyte levels the difference
between decimal and binary units approaches 10% as detailed later in this section. Given this difference it is
important to understand and use the expected unit for each particular value to maximize accuracy.
This document represents values using both decimal units and binary units. Values are represented by the
following formats:
- a) for decimal units: the value 3.5 terabytes is displayed as 3.5 TB (10^12);
- b) for binary units: the value 400 mebibytes per second is displayed as 400 MiB/sec (2^20)
- c) for an indication that all values in a row of a table are in specific units a statement is made in the left-most column:

Table 1 compares the names, symbols, and values of the binary and decimal units. Table 2 shows the increasing
percentage of difference between binary units and decimal units.

**Table 1 -- Comparison of binary and decimal units and values**

| Decimal Name | Symbol | Value (base-10) | Binary Name | Symbol | Value (base-2) |
|---|---|---|---|---|---|
| kilo | K | 10^3 | kibi | Ki | 2^10 |
| mega | M | 10^6 | mebi | Mi | 2^20 |
| giga | G | 10^9 | gibi | Gi | 2^30 |
| tera | T | 10^12 | tebi | Ti | 2^40 |
| peta | P | 10^15 | pebi | Pi | 2^50 |
| exa | E | 10^18 | exbi | Ei | 2^60 |

**Table 2 -- Percentage difference between binary and decimal units**

| Decimal Value | Binary Value | Percentage Difference |
|---|---|---|
| 100 kilobytes (KB) | 97.65 kibibytes (KiB) | 2.35% |
| 100 megabytes (MB) | 95.36 mebibytes (MiB) | 4.64% |
| 100 gigabytes (GB) | 93.13 gibibytes (GiB) | 6.87% |
| 100 terabytes (TB) | 90.94 tebibytes (TiB) | 9.06% |
| 100 petabytes (PB) | 88.81 pebibytes (PiB) | 11.19% |
| 100 exabytes (EB) | 86.73 exbibytes (EiB) | 13.27% |

### 2.2.4 Subpages

When pages have subpages (e.g., Mode Pages, Log Pages) the convention used for Page XXh Subpage YYh is
Page XXh[YYh].
When describing Security Protocol XXh with Security Protocol Specific YYYYh in the Security Protocol In
command or the Security Protocol Out command XXh[YYYYh] is used.

### 2.2.5 Hyperlinks

This document contains many hyperlinks. Every place the text says "see clause number" should be a hyperlink.
Hyperlinks have been given a special font to offset them from the rest of the text. That font is demonstrated in
this following link (see 2.2.5)

## 2.3 Tape Drive Model Names

From this section forward, through the remainder of this book, Tape Drive models are referred to collectively as
the LTO tape drive, the Ultrium tape drive or the 3580 tape drive. There are both a Full-High version and a
Half-High version, they are referred to as FH for Full-High and HH for Half-High. LTO drives are also available
with different host attachment interfaces, referred to as FC for Fibre Channel and SAS for Serially Attached
SCSI. LTO drives are also referred to by generation. Various combinations of these may be used where the
differences are meaningful and described in this document. Some examples include: LTO6, LTO7 FH, LTO8 HH
FC, etc.
