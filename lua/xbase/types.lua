---@class Project
---@field name string @Project Name or rather xproj generated file name.
---@field targets table<string, Target>
---@field root string @Project Root
---@field clients number[] @Project connected clients
---@field ignorePatterns string[]
---@field options ProjectOptions  @Options to override default behaviour
---@field packages table<string, ProjectPackage>

---@class Target
---@field type TargetType @Target Type
---@field platform Platform[] @Target Platform
---@field deploymentTarget table<Platform, string> @The deployment target
---@field configFiles table<string, string> @Config Files
---@field settings table<string, any> @Target specific build settings.
---@field dependencies ProjectDependency[] @Target Dependencies

---@class ProjectOptions
---@field bundleIdPrefix string
---@field carthageBuildPath string
---@field carthageExecutablePath string?
---@field defaultConfig string?
---@field deploymentTarget table<Platform, string>
---@field findCarthageFrameworks boolean
---@field settingPresets SettingPresets
---@field useBaseInternationalization boolean

---@class ProjectPackage
---@field url string?
---@field from string?
---@field branch string?
---@field exact_version string?
---@field major_version string?
---@field minor_version string?
---@field min_version string?
---@field max_version string?
---@field revision string?
---@field github string?
---@field path string?
---@field group string?

---@class Device
---@field availabilityError string?,
---@field dataPath string
---@field deviceTypeIdentifier string
---@field isAvailable boolean
---@field logPath string
---@field name string
---@field runtime_identifier string
---@field state string
---@field udid string

---@alias ProjectDependency table<string, string>

---@alias Platform
---| '"iOS"'
---| '"watchOS"'
---| '"tvOS"'
---| '"macOS"'

---@alias TargetType
---| '"application"'
---| '"application.on-demand-install-capable"'
---| '"application.messages"'
---| '"application.watchapp"'
---| '"application.watchapp2"'
---| '"app-extension"'
---| '"app-extension.intents-service"'
---| '"app-extension.messages"'
---| '"app-extension.messages-sticker-pack"'
---| '"bundle"'
---| '"bundle.ocunit-test"'
---| '"bundle.ui-testing"'
---| '"bundle.unit-test"'
---| '"framework"'
---| '"instruments-package"'
---| '"library.dynamic"'
---| '"library.static"'
---| '"framework.static"'
---| '"tool"'
---| '"tv-app-extension"'
---| '"watchapp2-container"'
---| '"watchkit-extension"'
---| '"watchkit2-extension"'
---| '"xcode-extension"'
---| '"driver-extension"'
---| '"system-extension"'
---| '"xpc-service"'

---@alias SettingPresets
---| '"all"'
---| '"project"'
---| '"targets"'
---| '"none"'
